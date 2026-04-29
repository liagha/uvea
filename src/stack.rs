use arrayvec::ArrayVec;

pub fn push<T, const N: usize>(stack: &mut ArrayVec<T, N>, value: T) {
    stack.try_push(value).expect("stack push overflow");
}

pub fn pop<T, const N: usize>(stack: &mut ArrayVec<T, N>) -> T {
    stack.pop().expect("stack pop underflow")
}