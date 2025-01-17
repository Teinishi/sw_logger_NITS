use crate::{
    range_check::{OutOfRangeError, RangeCheck},
    settings::Settings,
};
use serde::{Deserialize, Serialize};
use std::{
    cell::RefCell,
    collections::{BTreeMap, BTreeSet, HashMap, VecDeque},
    fs::File,
    io::{BufRead, BufReader, BufWriter, Write},
    path::Path,
    rc::Rc,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct QueueMaxLen<T> {
    vec: VecDeque<T>,
    max_len: usize,
}

impl<T> QueueMaxLen<T> {
    fn new() -> Self {
        Self::with_capacity(0)
    }

    fn with_capacity(max_len: usize) -> Self {
        Self {
            vec: VecDeque::new(),
            max_len,
        }
    }

    /*fn len(&self) -> usize {
        self.vec.len()
    }*/

    fn iter(&self) -> std::collections::vec_deque::Iter<'_, T> {
        self.vec.iter()
    }

    fn vec(&self) -> &VecDeque<T> {
        &self.vec
    }

    fn set_max_len(&mut self, max_len: usize) {
        let len = self.vec.len();
        if len < max_len {
            self.vec.reserve(max_len - len);
        } else if len > max_len {
            self.vec.drain(0..(len - max_len));
        }
        self.max_len = max_len;
    }

    fn push(&mut self, value: T) {
        let new_len = self.vec.len() + 1;
        if new_len > self.max_len {
            self.vec.drain(0..(new_len - self.max_len));
        }
        self.vec.push_back(value);
    }

    fn extend(&mut self, values: Vec<T>) {
        let new_len = self.vec.len() + values.len();
        if new_len > self.max_len {
            self.vec.drain(0..(new_len - self.max_len));
        }
        self.vec.extend(values);
    }

    fn back(&self) -> Option<&T> {
        self.vec.back()
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Serialize, Deserialize)]
pub struct NitsRelativeCarCount(i32); // 負の値が前方とする

impl NitsRelativeCarCount {
    pub fn get_channel_number(
        &self,
        car_count_front: u32,
        car_count_back: u32,
    ) -> Result<u32, OutOfRangeError<i32>> {
        let c = self.0;
        RangeCheck::new(c, (-15, true), (15, true))
            .check_result("NitsRelativeCarCount".to_string())?;
        RangeCheck::new(car_count_front as i32, (0, true), (15, false))
            .check_result("car_count_front".to_string())?;
        RangeCheck::new(car_count_front as i32, (0, true), (15, false))
            .check_result("car_count_back".to_string())?;

        if c < 0 {
            Ok(1 + car_count_front - c.unsigned_abs())
        } else if c > 0 {
            Ok(31 + c.unsigned_abs() - car_count_back)
        } else {
            Ok(16)
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

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub struct NitsCommand(u32);

impl NitsCommand {
    pub fn get_command_type(&self) -> u8 {
        (self.0 >> 24 & 0xFF).try_into().unwrap()
    }
    pub fn get_payload(&self) -> u32 {
        self.0 & 0xFFFFFF
    }
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct NitsTick {
    commonline: NitsCommand,
    commands: BTreeMap<NitsRelativeCarCount, NitsCommand>,
}

impl NitsTick {
    pub fn get_commonline(&self) -> &NitsCommand {
        &self.commonline
    }

    pub fn get_commands(&self) -> &BTreeMap<NitsRelativeCarCount, NitsCommand> {
        &self.commands
    }
}

#[derive(Debug, Deserialize)]
pub struct Values {
    values: BTreeMap<String, QueueMaxLen<f32>>,
    #[serde(skip)]
    settings: Rc<RefCell<Settings>>,
    nits_timeline: QueueMaxLen<NitsTick>,
    nits_senders: BTreeSet<NitsRelativeCarCount>,
    nits_command_types: BTreeSet<u8>,
}

impl Serialize for Values {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        #[derive(Serialize)]
        struct V {
            values: BTreeMap<String, QueueMaxLen<f32>>,
            nits_timeline: QueueMaxLen<NitsTick>,
            nits_senders: BTreeSet<NitsRelativeCarCount>,
            nits_command_types: BTreeSet<u8>,
        }

        if self.settings.borrow().keep_values {
            V {
                values: self.values.clone(),
                nits_timeline: self.nits_timeline.clone(),
                nits_senders: self.nits_senders.clone(),
                nits_command_types: self.nits_command_types.clone(),
            }
        } else {
            V {
                values: self
                    .values
                    .iter()
                    .map(|(k, _)| (k.clone(), QueueMaxLen::new()))
                    .collect(),
                nits_timeline: QueueMaxLen::new(),
                nits_senders: BTreeSet::new(),
                nits_command_types: BTreeSet::new(),
            }
        }
        .serialize(serializer)
    }
}

impl Values {
    pub fn new(settings: Rc<RefCell<Settings>>) -> Self {
        let max_len = settings.borrow().max_len();
        Self {
            values: BTreeMap::new(),
            settings,
            nits_timeline: QueueMaxLen::with_capacity(max_len),
            nits_senders: BTreeSet::new(),
            nits_command_types: BTreeSet::new(),
        }
    }

    pub fn set_settings(&mut self, settings: Rc<RefCell<Settings>>) {
        self.settings = settings;
    }

    pub fn set_max_len(&mut self) {
        let max_len = self.settings.borrow().max_len();

        for v in self.values.values_mut() {
            v.set_max_len(max_len);
        }
        self.nits_timeline.set_max_len(max_len);
        self.update_nits();
    }

    fn push(&mut self, key: String, values: Vec<f32>) {
        let max_len = self.settings.borrow().max_len();
        let v = self
            .values
            .entry(key)
            .or_insert_with(|| QueueMaxLen::with_capacity(max_len));
        v.extend(values);
    }

    pub fn add_data<S: std::hash::BuildHasher>(&mut self, data: HashMap<String, Vec<f32>, S>) {
        // NITS N01 から NITS N31 までの値を取得
        let mut nits_data: BTreeMap<u32, Vec<u32>> = BTreeMap::new();
        for i in 0..=31 {
            if let Some(channel) = data.get(&String::from(format!("NITS N{:02}", i))) {
                nits_data.insert(i, channel.iter().map(|v| v.to_bits()).collect());
            }
        }

        // NITS N32 (コモンライン) を取得し、他のチャンネルの値と時系列的に紐づける
        if let Some(n32) = data.get(&String::from("NITS N32")) {
            let len = n32.len();
            for (i, commonline_f) in n32.iter().enumerate() {
                let commonline = NitsCommand(commonline_f.to_bits());
                self.nits_command_types
                    .insert(commonline.get_command_type());
                let car_count_front = commonline.get_payload() & 15;
                let car_count_back = commonline.get_payload() >> 5 & 15;

                let mut commands: BTreeMap<NitsRelativeCarCount, NitsCommand> = BTreeMap::new();

                for j in -(car_count_front as i32)..=(car_count_back as i32) {
                    let key = NitsRelativeCarCount(j);
                    let channel_number = key.get_channel_number(
                        car_count_front.try_into().unwrap(),
                        car_count_back.try_into().unwrap(),
                    );
                    if let Ok(ch) = channel_number {
                        if let Some(channel) = nits_data.get(&ch) {
                            if let Some(c) = channel.get((i + channel.len()).saturating_sub(len)) {
                                let command = NitsCommand(*c);
                                self.nits_senders.insert(key);
                                self.nits_command_types.insert(command.get_command_type());
                                commands.insert(key, command);
                            }
                        }
                    }
                }

                self.nits_timeline.push(NitsTick {
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

    fn update_nits(&mut self) {
        // nits_senders と nits_command_types をリセット
        self.nits_senders = BTreeSet::new();
        self.nits_command_types = BTreeSet::new();
        for nits_tick in self.nits_timeline.iter() {
            self.nits_command_types
                .insert(nits_tick.commonline.get_command_type());
            for (sender, command) in &nits_tick.commands {
                self.nits_senders.insert(*sender);
                self.nits_command_types.insert(command.get_command_type());
            }
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
        match self.values.get(key) {
            Some(q) => Some(q.vec()),
            None => None,
        }
    }

    pub fn get_last_value_for_key(&self, key: &str) -> Option<f32> {
        self.values
            .get(key)
            .as_ref()
            .and_then(|v| v.back())
            .cloned()
    }

    pub fn get_nits_timeline(&self) -> &VecDeque<NitsTick> {
        &self.nits_timeline.vec()
    }

    pub fn get_nits_senders(&self) -> &BTreeSet<NitsRelativeCarCount> {
        &self.nits_senders
    }

    pub fn get_nits_command_types(&self) -> &BTreeSet<u8> {
        &self.nits_command_types
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
