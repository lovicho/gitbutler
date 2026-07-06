import { getButtonClassName } from "#ui/components/Button.tsx";
import { Component, type ReactNode } from "react";
import styles from "./WorkspacePage.module.css";

export class WorkspacePageErrorBoundary extends Component<
	{
		onReset: () => void;
		children: ReactNode;
	},
	{
		error: Error | null;
	}
> {
	state = { error: null as Error | null };

	static getDerivedStateFromError(error: unknown) {
		return {
			error:
				error instanceof Error
					? error
					: new Error(typeof error === "string" ? error : JSON.stringify(error)),
		};
	}

	handleRetry(): void {
		this.props.onReset();
		this.setState({ error: null });
	}

	render(): ReactNode {
		if (!this.state.error) return this.props.children;

		return (
			<div className={styles.error}>
				<h1 className={styles.errorTitle}>Something went wrong.</h1>
				<div className={styles.errorActions}>
					<button
						type="button"
						className={getButtonClassName({})}
						onClick={() => this.handleRetry()}
					>
						Retry
					</button>
				</div>
				<code className={styles.errorMessage}>{this.state.error.message}</code>
			</div>
		);
	}
}
