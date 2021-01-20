use frame_support::{StorageMap, Parameter};
use sp_runtime::traits::Member;
use codec::{Encode, Decode};

#[cfg_attr(feature = "std", derive(Debug, PartialEq, Eq))]
#[derive(Encode, Decode)]
pub struct LinkedItem<Value> {
    pub prev: Option<Value>,
    pub next: Option<Value>,
}

pub struct LinkedList<Storage, Key, Value>(sp_std::marker::PhantomData<(Storage, Key, Value)>);

impl<Storage, Key, Value> LinkedList<Storage, Key, Value> where
    Value: Parameter + Member + Copy,
    Key: Parameter,
    Storage: StorageMap<(Key, Option<Value>), LinkedItem<Value>, Query=Option<LinkedItem<Value>>>,
{
    fn read_head(key: &Key) -> LinkedItem<Value> {
        Self::read(key, None)
    }

    fn write_head(account: &Key, item: LinkedItem<Value>) {
        Self::write(account, None, item);
    }

    fn read(key: &Key, value: Option<Value>) -> LinkedItem<Value> {
        Storage::get((&key, value)).unwrap_or_else(|| LinkedItem {
            prev: None,
            next: None,
        })
    }

    fn write(key: &Key, value: Option<Value>, item: LinkedItem<Value>) {
        Storage::insert((&key, value), item);
    }

    pub fn append(key: &Key, value: Value) {
        // 作业
        // 双向循环链表
        let mut head_node: LinkedItem<Value> = Self::read_head(key); // 头节点
        match head_node.next {
            None => { // 没有其他节点存在
                // 节点prev、next指向新节点
                head_node.next = Some(value);
                head_node.prev = Some(value);
                let new_item: LinkedItem<Value> = LinkedItem {
                    prev: None,
                    next: None,
                };
                Self::write_head(key, head_node); // 更新节点
                Self::write(key, Some(value), new_item); // 增加新节点
            }
            Some(_) => { // 存在下一个节点
                let mut last_node: LinkedItem<Value> = Self::read(key, head_node.prev); // 最后一个节点
                let new_item: LinkedItem<Value> = LinkedItem { // 插入的新节点
                    prev: head_node.prev, // 前驱指向头节点的prev，即尾
                    next: last_node.next, // 后继指向尾节点的后一个节点，即头
                };
                last_node.next = Some(value);
                head_node.prev = Some(value);
                Self::write_head(key, head_node); // 更新节点
                Self::write(key, new_item.prev, last_node); // 更新节点
                Self::write(key, Some(value), new_item); // 加入新节点
            }
        }
    }

    pub fn remove(key: &Key, value: Value) {
        // 作业
        let del_node: LinkedItem<Value> = Self::read(key, Some(value));
        if let None = del_node.prev {
            if let None = del_node.next{
                return;
            }
        }
        let mut del_prev_node:LinkedItem<Value> = Self::read(key, del_node.prev); // 删除节点的前一个节点
        let mut del_next_node:LinkedItem<Value> = Self::read(key, del_node.next); // 删除节点的后一个节点
        del_prev_node.next = del_node.next;
        del_next_node.prev = del_node.prev;
        Self::write(key, del_node.prev, del_prev_node);
        Self::write(key, del_node.next, del_next_node);
        Storage::remove((key, Some(value)));
    }
}
