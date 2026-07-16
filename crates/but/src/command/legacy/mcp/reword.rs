use but_action::reword::RewordInput;
use but_ctx::ThreadSafeContext;
use but_llm::LLMProvider;

#[derive(Debug, Clone)]
struct Job {
    ctx: ThreadSafeContext,
    input: RewordInput,
}

#[derive(Debug, Clone)]
pub struct Handler {
    sender: Option<tokio::sync::mpsc::UnboundedSender<Job>>,
}

impl Handler {
    pub fn new_in_background() -> Self {
        let sender = LLMProvider::default_openai()
            .map(|llm| {
                let (sender, mut receiver) = tokio::sync::mpsc::unbounded_channel::<Job>();
                tokio::task::spawn(async move {
                    while let Some(job) = receiver.recv().await {
                        let mut ctx = job.ctx.into_thread_local();
                        let context_lines = ctx.settings.context_lines;
                        let commit_id = job.input.commit_id;
                        let result = (|| -> anyhow::Result<_> {
                            let mut meta = ctx.meta()?;
                            let (_guard, repo, mut ws, _db) = ctx.workspace_mut_and_db_mut()?;
                            but_action::reword::commit(
                                &llm,
                                job.input,
                                &repo,
                                &mut ws,
                                &mut meta,
                                context_lines,
                            )
                        })();
                        if let Err(err) = result {
                            tracing::warn!(?err, %commit_id, "Failed to reword commit in background");
                        }
                    }
                });
                Some(sender)
            })
            .unwrap_or_default();

        Self { sender }
    }

    pub fn queue(&self, ctx: ThreadSafeContext, input: RewordInput) {
        if let Some(sender) = &self.sender
            && sender.send(Job { ctx, input }).is_err()
        {
            tracing::warn!("Failed to queue commit reword because the background worker stopped");
        }
    }
}
