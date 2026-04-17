use crate::state::State;

struct FeedbackData {
    skip_offset: usize,
    marquee_overflow: f32,
}

pub struct Feedback {
    skip_offset: usize,
    marquee_overflow: f32,
    tx: Option<oneshot::Sender<FeedbackData>>,
}

impl Feedback {
    pub fn report_skip_offset(&mut self, skip_offset: usize) {
        self.skip_offset = skip_offset;
    }

    pub fn report_marquee_overflow(&mut self, overflow: f32) {
        self.marquee_overflow = overflow;
    }
}

impl Drop for Feedback {
    fn drop(&mut self) {
        if let Some(tx) = self.tx.take() {
            let _ = tx.send(FeedbackData {
                skip_offset: self.skip_offset,
                marquee_overflow: self.marquee_overflow,
            });
        }
    }
}

pub struct FeedbackListener {
    rx: oneshot::Receiver<FeedbackData>,
}

impl FeedbackListener {
    pub fn apply(self, state: &mut State) {
        let data = self.rx.recv().unwrap();
        state.update_skip_offset(data.skip_offset);
        state.set_marquee_overflow(data.marquee_overflow);
    }
}

pub fn channel() -> (Feedback, FeedbackListener) {
    let (tx, rx) = oneshot::channel();
    (
        Feedback {
            skip_offset: 0,
            marquee_overflow: 0.0,
            tx: Some(tx),
        },
        FeedbackListener { rx },
    )
}
