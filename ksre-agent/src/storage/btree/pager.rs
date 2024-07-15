use std::fs::File;
use std::os::unix::fs::FileExt;
use std::{u64, usize};

use super::constant::DEFAULT_PAGE_SIZE;

#[derive(Default, Debug)]
pub struct Page {
    pub data: Vec<u8>,
    pub page_number: u64,
}

impl Page {
    #[inline]
    pub fn new_empty_with_pn(page_size: usize, page_number: u64) -> Page {
        Page {
            data: vec![0u8; page_size],
            page_number,
        }
    }
}

// Pager manager
pub struct Pager {
    file: File,
    page_size: usize,
}

// Pager[#TODO] (should add some comments)
impl Pager {
    pub fn new(file: File) -> Self {
        Pager {
            file,
            page_size: DEFAULT_PAGE_SIZE,
        }
    }

    pub fn allocate_page(&self, page_number: u64) -> Page {
        Page {
            data: vec![0; self.page_size],
            page_number,
        }
    }

    pub fn write_page(&self, page: &Page) {
        let offset = page.page_number * DEFAULT_PAGE_SIZE as u64;
        self.file
            .write_at(page.data.as_ref(), offset)
            .expect("write page failed");
    }

    pub fn read_page(&self, page_number: u64) -> Option<Page> {
        let mut new_page = Page::new_empty_with_pn(self.page_size, page_number);
        let offset = new_page.page_number * DEFAULT_PAGE_SIZE as u64;
        if self.file.read_exact_at(&mut new_page.data, offset).is_ok() {
            Some(new_page)
        } else {
            None
        }
    }
}
