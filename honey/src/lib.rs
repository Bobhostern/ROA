///# Why create honey?
///# I want to find a solution to rodio's slightly choppy input.
///# This won't be immediately developed, but once we get to sounds and stuff,
///# here we go!

///# The first 2 formats we'll support are wav (through hound) and flac (through clavox)
///# Our secret is that we <<buffer>> everything, even a little bit, at least enough for
///# 5 frames of sound in the future.
#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}
