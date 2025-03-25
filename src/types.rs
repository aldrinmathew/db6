use std::{
    fmt::Display,
    sync::{Arc, LazyLock, Mutex},
};

static ID_COUNTER: LazyLock<Arc<Mutex<Vec<u64>>>> = LazyLock::new(|| Arc::new(Mutex::new(vec![0])));

pub struct ID {
    id: Vec<u64>,
}

impl ID {
    pub fn new() -> ID {
        let mut id_lock = ID_COUNTER.try_lock();
        while id_lock.is_err() {
            id_lock = ID_COUNTER.try_lock();
        }
        let vec_val = id_lock.as_mut().unwrap();
        if *vec_val.last().unwrap() == u64::MAX {
            vec_val.push(0);
        } else {
            *vec_val.last_mut().unwrap() += 1;
        }
        return ID {
            id: vec_val.clone(),
        };
    }
}

impl PartialEq for ID {
    fn eq(&self, other: &Self) -> bool {
        if self.id.len() != other.id.len() {
            return false;
        }
        for i in 0..self.id.len() {
            if self.id[i] != other.id[i] {
                return false;
            }
        }
        return true;
    }

    fn ne(&self, other: &Self) -> bool {
        if self.id.len() != other.id.len() {
            return true;
        }
        for i in 0..self.id.len() {
            if self.id[i] != other.id[i] {
                return true;
            }
        }
        return false;
    }
}

impl Eq for ID {}

impl Display for ID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for i in 0..self.id.len() {
            f.write_str(self.id[i].to_string().as_str())?;
            if i != self.id.len() - 1 {
                f.write_str("-")?;
            }
        }
        Ok(())
    }
}
