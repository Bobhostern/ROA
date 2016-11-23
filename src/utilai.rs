// From GDC2010 "Improving AI Decision Modeling Through Utility Theory"
type Appraisal = f32;

// QUESTION: Do we add
pub struct Behavior<C> {
    considerations: Vec<Box<Consideration<C>>>,
}

impl<C> Behavior<C> {
    pub fn new() -> Behavior<C> {
        Behavior {
            considerations: vec![]
        }
    }

    pub fn add_consideration(&mut self, c: Box<Consideration<C>>) {
        self.considerations.push(c)
    }

    // Returns the average utility calculated by all Considerations, limited to the range 0.0-1.0
    pub fn evaluate(&self, c: &C) -> f32 {
        let appraisals = self.considerations.iter().map(|x| x.evaluate(c)).collect::<Vec<_>>();
        let base = appraisals.iter().fold(0.0, |a, x| a + x);
        let fina = base / appraisals.len() as f32;
        // A Behavior that considers nothing is a logic error, but we let it slide as useless
        fina.max(0.0).min(1.0)
    }
}

// C is the Context of this consideration. I don't feel comfortable hard-lining a certain context,
// so now these classes are generic over context
pub trait Consideration<C> {
    fn evaluate(&self, &C) -> Appraisal;
}
