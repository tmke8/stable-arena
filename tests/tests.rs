use stable_arena::{declare_arena, IsCopy, IsNotCopy};

#[test]
fn test_declare_arena() {
    declare_arena!([
        ints: i32,
        boxes: Box<i32>,
    ]);

    let arena = Arena::default();
    let num = arena.alloc::<_, IsCopy>(1);
    assert_eq!(num, &1);
    let boxed = arena.alloc::<_, IsNotCopy>(Box::new(2));
    assert_eq!(boxed, &Box::new(2));
}
