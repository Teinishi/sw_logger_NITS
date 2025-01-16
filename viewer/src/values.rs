use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet, HashMap, VecDeque},
    fs::File,
    io::{BufRead, BufReader, BufWriter, Write},
    path::Path,
};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Serialize, Deserialize)]
pub struct NitsRelativeCarCount(isize); // 負の値が前方とする

impl NitsRelativeCarCount {
    pub fn get_channel_number(&self, car_count_front: usize, car_count_back: usize) -> usize {
        // TODO: 各引数の範囲チェック
        if self.0 < 0 {
            (1 + car_count_front).saturating_sub(self.0.unsigned_abs())
        } else if self.0 > 0 {
            (31 + self.0.unsigned_abs()).saturating_sub(car_count_back)
        } else {
            16
        }
    }
}

impl std::fmt::Display for NitsRelativeCarCount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.0 < 0 {
            write!(f, "{} Front", (-self.0).to_string())
        } else if self.0 > 0 {
            write!(f, "{} Back", self.0.to_string())
        } else {
            write!(f, "Self")
        }
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct NitsTick {
    pub commonline: u32,
    pub commands: BTreeMap<NitsRelativeCarCount, u32>,
}

#[derive(Debug, PartialEq, Deserialize)]
pub struct Values {
    values: BTreeMap<String, VecDeque<f32>>,
    max_len: usize,
    pub nits_timeline: VecDeque<NitsTick>,
    pub nits_senders: BTreeSet<NitsRelativeCarCount>,
}

impl Serialize for Values {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        #[derive(Serialize)]
        struct V {
            values: BTreeMap<String, Vec<f32>>,
            max_len: usize,
        }
        V {
            values: self
                .values
                .iter()
                .map(|(k, _)| (k.clone(), vec![]))
                .collect(),
            max_len: self.max_len,
        }
        .serialize(serializer)
    }
}

impl Default for Values {
    fn default() -> Self {
        Self::with_capacity(3600)
    }
}

impl Values {
    pub fn with_capacity(max_len: usize) -> Self {
        Self {
            values: BTreeMap::new(),
            max_len,
            nits_timeline: VecDeque::new(),
            nits_senders: BTreeSet::new(),
        }
    }

    pub fn max_len(&self) -> usize {
        self.max_len
    }

    pub fn set_max_len(&mut self, max_len: usize) {
        self.max_len = max_len;
        for v in self.values.values_mut() {
            if v.len() < max_len {
                v.reserve(max_len - v.len());
            }
            if v.len() > max_len {
                v.drain(0..(v.len() - max_len));
            }
        }
    }

    fn push(&mut self, key: String, values: Vec<f32>) {
        let v = self
            .values
            .entry(key)
            .or_insert_with(|| VecDeque::with_capacity(self.max_len));
        if v.len() + values.len() > self.max_len {
            v.drain(0..(v.len() + values.len() - self.max_len));
        }
        v.extend(values);
    }

    pub fn add_data<S: std::hash::BuildHasher>(&mut self, data: HashMap<String, Vec<f32>, S>) {
        // NITS N01 から NITS N31 までの値を取得
        let mut nits_data: BTreeMap<usize, Vec<u32>> = BTreeMap::new();
        for i in 0..=31 {
            if let Some(channel) = data.get(&String::from(format!("NITS N{:02}", i))) {
                nits_data.insert(i, channel.iter().map(|v| v.to_bits()).collect());
            }
        }

        // NITS N32 (コモンライン) を取得し、他のチャンネルの値と時系列的に紐づける
        if let Some(n32) = data.get(&String::from("NITS N32")) {
            let len = n32.len();
            for (i, commonline_f) in n32.iter().enumerate() {
                let commonline = commonline_f.to_bits();
                let car_count_front = commonline & 15;
                let car_count_back = commonline >> 5 & 15;

                let mut commands: BTreeMap<NitsRelativeCarCount, u32> = BTreeMap::new();

                for j in -(car_count_front as isize)..=(car_count_back as isize) {
                    let key = NitsRelativeCarCount(j);
                    let channel_number = key.get_channel_number(
                        car_count_front.try_into().unwrap(),
                        car_count_back.try_into().unwrap(),
                    );
                    if let Some(channel) = nits_data.get(&channel_number) {
                        if let Some(command) = channel.get((i + channel.len()).saturating_sub(len))
                        {
                            commands.insert(key, command.clone());
                            self.nits_senders.insert(key);
                        }
                    }
                }

                let drain = (self.nits_timeline.len() + 1).saturating_sub(self.max_len);
                if drain > 0 {
                    self.nits_timeline.drain(0..drain);
                }
                self.nits_timeline.push_back(NitsTick {
                    commonline,
                    commands,
                });
            }
        }

        // NITSに限らない通常のデータの処理
        for (k, v) in data {
            self.push(k, v);
        }
    }

    pub fn contains_key(&self, key: &str) -> bool {
        self.values.contains_key(key)
    }

    pub fn keys(&self) -> impl Iterator<Item = &String> {
        self.values.keys()
    }

    pub fn iter_for_key(
        &self,
        key: &str,
    ) -> Option<impl Iterator<Item = &f32> + ExactSizeIterator + DoubleEndedIterator> {
        self.values.get(key).map(|v| v.iter())
    }

    pub fn values_for_key(&self, key: &str) -> Option<&VecDeque<f32>> {
        self.values.get(key)
    }

    pub fn get_last_value_for_key(&self, key: &str) -> Option<f32> {
        self.values
            .get(key)
            .as_ref()
            .and_then(|v| v.back())
            .cloned()
    }

    pub fn load_csv<P: AsRef<Path>>(&mut self, file_path: P) {
        if let Ok(file) = File::open(file_path) {
            let mut first_row: Option<Vec<String>> = None;

            for result in BufReader::new(file).lines() {
                if let Ok(l) = result {
                    let row = l.split(',');

                    if let Some(ref keys) = first_row {
                        let mut data = HashMap::new();
                        for (key, v) in keys.iter().zip(row) {
                            data.insert(key.clone(), vec![v.parse::<f32>().unwrap()]);
                        }
                        self.add_data(data);
                    } else {
                        first_row = Some(row.into_iter().map(|s| String::from(s)).collect());
                    }
                }
            }
        }
    }

    pub fn save_csv<'a, K>(&self, path: &Path, keys: K) -> Result<(), std::io::Error>
    where
        K: Iterator<Item = &'a String>,
    {
        let mut writer = BufWriter::new(File::create(path)?);
        let mut values = Vec::with_capacity(self.values.len());
        let mut first = true;
        let mut max_len = 0;
        for key in keys {
            if let Some(v) = self.values_for_key(key) {
                if first {
                    first = false
                } else {
                    writer.write_all(",".as_bytes())?;
                }
                writer.write_all(key.as_bytes())?;
                max_len = max_len.max(v.len());
                values.push(v);
            }
        }
        writer.write_all("\n".as_bytes())?;
        for index in 0..max_len {
            for (i, vec) in values.iter().enumerate() {
                let offset = max_len - vec.len();
                if offset > index {
                    writer.write_all(",".as_bytes())?;
                    continue;
                }
                if let Some(v) = vec.get(index - offset) {
                    if i == 0 {
                        writer.write_fmt(format_args!("{}", v))?;
                    } else {
                        writer.write_fmt(format_args!(",{}", v))?;
                    }
                } else {
                    writer.write_all(",".as_bytes())?;
                }
            }
            writer.write_all("\n".as_bytes())?;
        }
        writer.flush()?;
        Ok(())
    }
}
