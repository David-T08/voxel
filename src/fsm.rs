pub trait StateLifecycle<C, Cmd> {
    fn on_enter(&mut self, _context: &C, _out: &mut Vec<Cmd>) {}
    fn on_exit(&mut self, _context: &C, _out: &mut Vec<Cmd>) {}
}

pub enum Transition<S> {
    Stay,
    Switch(S),
}

pub trait StateUpdate<Context, Cmd>: Sized {
    fn update(&mut self, delta: f32, context: &Context, out: &mut Vec<Cmd>) -> Transition<Self>;
}

#[derive(Debug)]
pub struct StateMachine<S> {
    state: S,
}

impl<S> StateMachine<S> {
    pub fn new(initial: S) -> Self {
        Self { state: initial }
    }

    pub fn state(&self) -> &S {
        &self.state
    }

    pub fn tick<Context, Cmd>(&mut self, delta: f32, context: &Context) -> Vec<Cmd>
    where
        S: StateUpdate<Context, Cmd> + StateLifecycle<Context, Cmd>,
    {
        let mut out = Vec::new();

        let transition = self.state.update(delta, context, &mut out);
        if let Transition::Switch(mut next) = transition {
            self.state.on_exit(context, &mut out);
            next.on_enter(context, &mut out);
            self.state = next;
        }

        out
    }
}
