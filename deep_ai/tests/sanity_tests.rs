extern crate deep_ai;
use deep_ai::normal_nets::lolok;

#[test]
fn lol_ok() {
    assert_eq!(lolok(), lolok());
}


#[test]
fn vec_eq() {
    assert_eq!(vec![1., 2., 3.], vec![1., 2., 3.]);
    assert_ne!(vec![1.1, 2., 3.], vec![1., 2., 3.]);

}
