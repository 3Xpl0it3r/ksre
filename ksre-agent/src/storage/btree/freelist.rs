#[derive(Default, Debug)]
pub struct Freelist {
    pub max_page: u64, // 8B
    pub released_pages: Vec<u64>,
}

// Freelist[#TODO] (should add some comments)
impl Freelist {
    pub fn get_next_page(&mut self) -> u64 {
        // first page reserverd for metadata page, is hard code set to 0 in constant
        self.released_pages.pop().unwrap_or_else(|| {
            self.max_page += 1;
            self.max_page
        })
    }

    pub fn release_page(&mut self, _page_number: u64) {
        // todo 可能需要多个页面来存放released page
        /* self.released_pages.push(page_number) */
    }

    pub fn serialize(&self, buffer: &mut [u8]) {
        let mut offset = 0;

        buffer[offset..offset + 8].clone_from_slice(u64::to_le_bytes(self.max_page).as_ref());
        offset += 8;

        let freeed_cnt = self.released_pages.len();
        buffer[offset..offset + 2].clone_from_slice(u16::to_le_bytes(freeed_cnt as u16).as_ref());
        offset += 2;

        for i in 0..freeed_cnt {
            let element = self.released_pages[i];
            buffer[offset..offset + 8].clone_from_slice(u64::to_le_bytes(element).as_ref());
            offset += 8;
        }
    }

    pub fn deserialize(&mut self, buffer: &[u8]) {
        if buffer.len() < 2 {
            return;
        }
        let mut offset = 0;

        self.max_page = u64::from_le_bytes(buffer[offset..offset + 8].try_into().unwrap());
        offset += 8;

        // get freed cnt
        let freeed_cnt = u16::from_le_bytes(buffer[offset..offset + 2].try_into().unwrap());
        offset += 2;
        if freeed_cnt == 0 {
            return;
        }

        for _ in 0..freeed_cnt {
            let element = u64::from_le_bytes(buffer[offset..offset + 8].try_into().unwrap());
            self.released_pages.push(element);
            offset += 8;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn test_freelist_basic_operations() {
        let free_size = |freelist: &Freelist| -> usize {
            8 + // max_page
        2 + // released_pages计数
        freelist.released_pages.len() * 8 // 每个released页面的大小
        };
        // 创建一个Freelist实例
        let mut freelist = Freelist {
            max_page: 0,
            released_pages: Vec::new(),
        };

        // 初始化时max_page应该为0且released_pages为空
        assert_eq!(freelist.max_page, 0);
        assert!(freelist.released_pages.is_empty());

        // 添加并获取一页
        freelist.release_page(100);
        assert_eq!(freelist.get_next_page(), 100);

        // 再次获取一页，此时released_pages为空，应增加max_page
        assert_eq!(freelist.get_next_page(), 1);

        // 释放多页并验证序列化和反序列化
        let pages_to_release = vec![300, 400, 500];
        for page in pages_to_release.iter() {
            freelist.release_page(*page);
        }

        // 序列化到字节数组
        let mut serialization_buffer = vec![0; free_size(&freelist)];
        freelist.serialize(&mut serialization_buffer[..]);

        // 反序列化回一个新的Freelist实例
        let mut deserialized_freelist = Freelist::default();
        deserialized_freelist.deserialize(&serialization_buffer);

        // 验证反序列化的Freelist与原Freelist状态一致
        assert_eq!(deserialized_freelist.max_page, freelist.max_page);
        assert_eq!(
            deserialized_freelist.released_pages,
            freelist.released_pages
        );

        // 继续测试get_next_page函数在反序列化后是否正常工作
        for expected_page in pages_to_release.iter().rev() {
            assert_eq!(deserialized_freelist.get_next_page(), *expected_page);
        }
        assert_eq!(deserialized_freelist.get_next_page(), freelist.max_page + 1);
    }

    fn test_freelist_basic_functionality() {
        let mut freelist = Freelist::default();
        assert_eq!(freelist.get_next_page(), 1); // 第一个获取的页面应该是1
        freelist.release_page(1); // 释放页面1
        assert_eq!(freelist.get_next_page(), 1); // 再次获取应该还是页面1
        freelist.release_page(2); // 尝试释放一个未分配的页面，不会有影响
    }

    fn test_freelist_serialization_deserialization() {
        let mut freelist = Freelist::default();
        freelist.get_next_page(); // 获取页面1
        freelist.get_next_page(); // 获取页面2
        freelist.release_page(2); // 释放页面2

        let mut buffer = vec![0; 20]; // 准备一个足够大的缓冲区
        freelist.serialize(&mut buffer); // 序列化到缓冲区
                                         // allocated=2, freelist =[2]

        let mut deserialized_freelist = Freelist::default();
        deserialized_freelist.deserialize(&buffer); // 从缓冲区反序列化

        // should 2
        assert_eq!(deserialized_freelist.get_next_page(), 2); // 应该能够恢复到获取页面1的状态
                                                              // should 3
        assert_eq!(deserialized_freelist.get_next_page(), 3); // 然后是页面2
                                                              // should 3
        assert_eq!(deserialized_freelist.get_next_page(), 4); // 接下来是新分配的页面3
    }

    fn test_freelist_boundary_conditions() {
        let mut freelist = Freelist::default();
        for _ in 0..10 {
            let page = freelist.get_next_page(); // 获取10个页面
            freelist.release_page(page); // 并立即释放
        }

        for page in 1..=10 {
            assert_eq!(freelist.get_next_page(), page); // 应该按顺序获取页面1到10
        }

        assert_eq!(freelist.get_next_page(), 11); // 接下来应该是新分配的页面11
    }

    fn test_freelist_exception_handling() {
        let mut freelist = Freelist::default();
        freelist.get_next_page(); // 获取页面1
        freelist.get_next_page(); // 获取页面2

        // 故意提供一个不完整的序列化数据
        let incomplete_buffer = vec![0; 10]; // 缺少了释放页面的数量信息
        freelist.deserialize(&incomplete_buffer); // 尝试反序列化

        assert_eq!(freelist.get_next_page(), 1); // 由于数据不完整，应该恢复到初始状态
        assert_eq!(freelist.get_next_page(), 2);
    }
}
