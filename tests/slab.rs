use ruyi_slab::Slab;

#[test]
fn slab_insert() {
    let mut slab = Slab::with_capacity(1);
    slab.insert(10);
    assert_eq!(slab.len(), 1);

    slab.insert(20);
    assert_eq!(slab.len(), 2);

    slab.insert(30);
    assert_eq!(slab.len(), 3);
}

#[test]
fn slab_remove() {
    let mut slab = Slab::new();
    let a1 = slab.insert(10);
    let a2 = slab.insert(20);

    let f1 = slab.remove(a1);
    assert!(f1.is_some());
    assert_eq!(f1.unwrap(), 10);

    slab.insert(30);

    slab.remove(a2);

    assert_eq!(slab.len(), 1);

    assert!(slab.remove(slab.len()).is_none());
}

#[test]
fn slab_get() {
    let mut slab = Slab::new();
    let a1 = slab.insert(10);
    let a2 = slab.insert(20);

    assert_eq!(slab[a1], 10);
    assert_eq!(slab[a2], 20);

    *slab.get_mut(a2).unwrap() = 200;
    assert_eq!(slab[a2], 200);
    slab[a2] = 40;
    assert_eq!(slab.remove(a2).unwrap(), 40);

    let a3 = slab.insert(30);
    assert_eq!(slab[a3], 30);

    unsafe {
        assert_eq!(*slab.get_unchecked(a3), 30);
        *slab.get_unchecked_mut(a3) = 300;
    }
    assert_eq!(slab[a3], 300);

    assert_eq!(slab.remove(a3).unwrap(), 300);
    assert_eq!(slab.remove(a1).unwrap(), 10);
}

#[test]
fn slab_remove_unchecked() {
    let mut slab = Slab::new();
    let a1 = slab.insert(10);
    slab.insert(20);

    let c1 = unsafe { slab.remove_unchecked(a1) };
    assert_eq!(c1, 10);

    let a3 = slab.insert(30);

    let c3 = unsafe { slab.remove_unchecked(a3) };
    assert_eq!(c3, 30);

    assert_eq!(slab.len(), 1);
}
