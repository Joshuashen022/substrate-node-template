# Ouroboros Autosyn Code Logic
____
##Main Logic
____
```
// Get slot duration
read buffer for data
calculate slot duration
inner_delay.await;

// get slot info
let info = SlotInfo::new();
slots.next_slot(info);
check_header(info);

//Other function;
worker.on_slot(slot_info).await;
if leader
  generate Adjust
```


**Node** contained information

###Get slot duration
Buffer for slot information




###Other
Buffer
```rust
vec!(Adjust, Adjust, Adjust,);
```


Send Message
```rust
pub struct Adjust{
    block_last: Block,
    receive_time: TimeStamp,
    proof: LeaderProof,
}
```





