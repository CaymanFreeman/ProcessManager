use crate::processes::data;

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Eq)]
pub enum SortCategory {
    Id,
    Memory,
    Cpu,
    DiskRead,
    DiskWrite,
    Status,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub enum SortDirection {
    Ascending,
    Descending,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct SortMethod {
    pub(crate) category: SortCategory,
    pub(crate) direction: SortDirection,
}

impl SortMethod {
    pub fn sort(&self, processes_info: &mut [data::ProcessInfo]) {
        match self.category {
            SortCategory::Id => processes_info.sort_by(|a, b| a.id.cmp(&b.id)),
            SortCategory::Memory => processes_info.sort_by(|a, b| a.memory.cmp(&b.memory)),
            SortCategory::Cpu => processes_info.sort_by(|a, b| a.cpu.cmp(&b.cpu)),
            SortCategory::DiskRead => processes_info.sort_by(|a, b| a.disk_read.cmp(&b.disk_read)),
            SortCategory::DiskWrite => {
                processes_info.sort_by(|a, b| a.disk_write.cmp(&b.disk_write));
            }
            SortCategory::Status => processes_info.sort_by(|a, b| a.status.cmp(&b.status)),
        }

        if matches!(self.direction, SortDirection::Descending) {
            processes_info.reverse();
        }
    }

    pub fn toggle_direction(&mut self) {
        match self.direction {
            SortDirection::Ascending => self.direction = SortDirection::Descending,
            SortDirection::Descending => self.direction = SortDirection::Ascending,
        }
    }
}

impl Default for SortMethod {
    fn default() -> Self {
        Self {
            category: SortCategory::Cpu,
            direction: SortDirection::Descending,
        }
    }
}
