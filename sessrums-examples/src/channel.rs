


struct Channel<T,IO> {
    fn send(&self, msg: T) -> Result<(), Channel>;
    fn recv(&self) -> Result<T, Channel>;
}

