#[cfg(test)]
pub mod fixture_core {
    use crate::declarations::errors::ImmuxResult;
    use crate::storage::core::CoreStore;
    use crate::storage::instructions::{Answer, Instruction};

    type BoxedRespondFn<'a> = Box<dyn Fn(&Instruction) -> ImmuxResult<Answer> + 'a>;

    pub struct FixtureCore<'a> {
        respond: BoxedRespondFn<'a>,
    }

    impl<'a> FixtureCore<'a> {
        pub fn new(respond: BoxedRespondFn<'a>) -> Self {
            FixtureCore { respond }
        }
    }

    impl<'a> CoreStore for FixtureCore<'a> {
        fn execute(&mut self, instruction: &Instruction) -> ImmuxResult<Answer> {
            return (self.respond)(instruction);
        }
    }
}
