import { Toast, type ToastManager } from "@base-ui/react";
import type { FC } from "react";
import { classes } from "./classes.ts";
import type { IconName } from "./iconNames.ts";
import { Snackbar, type SnackbarVariant } from "./Snackbar.tsx";
import styles from "./Snackbars.module.css";

/**
 * What a snackbar says about how the news landed, passed to `add({ type })`: a Snackbar variant, or
 * `loading` for work still running, which shows a spinner, has no timer, and is updated to its
 * result once the work is done.
 *
 * @public
 */
export type SnackbarType = SnackbarVariant | "loading";

/**
 * What a snackbar carries in Base UI's `data`: `add({ data: { icon, dismissable } })`.
 *
 * @public
 */
export type SnackbarData = {
	/** Overrides the variant's own glyph. */
	icon?: IconName;
	/** Gives it a close button. Pair it with `timeout: 0` for one that waits to be dealt with. */
	dismissable?: boolean;
};

const variant = (type: string | undefined): SnackbarVariant =>
	type === "warning" || type === "danger" || type === "safe" ? type : "info";

const Stack: FC<{ className?: string }> = ({ className }) => {
	const { toasts, close } = Toast.useToastManager<SnackbarData>();
	return (
		<Toast.Portal>
			<Toast.Viewport className={classes(styles.viewport, className)}>
				{toasts.map((snackbar, index) => (
					<Toast.Root
						key={snackbar.id}
						toast={snackbar}
						className={styles.root}
						// A leaving snackbar keeps its place in the list but not its visible index, so this is
						// how the stylesheet tells the front one, which sinks out, from the rest.
						data-front={index === 0 || undefined}
						// On the root rather than the snackbar: the root captures the pointer for swiping, so a
						// click lands here.
						onClick={() => close(snackbar.id)}
					>
						<Snackbar
							// The root announces it, through the title; the snackbar itself stays quiet.
							role="none"
							variant={variant(snackbar.type)}
							icon={snackbar.type === "loading" ? "spinner" : snackbar.data?.icon}
							onDismiss={snackbar.data?.dismissable ? () => close(snackbar.id) : undefined}
						>
							<Toast.Title render={<span />} />
						</Snackbar>
					</Toast.Root>
				))}
			</Toast.Viewport>
		</Toast.Portal>
	);
};

/**
 * Where snackbars appear: a stack at the bottom centre of the window. Render it once with a manager
 * of its own, separate from the toasts', from Base UI's `Toast.createToastManager<SnackbarData>()`.
 * Raise one with `manager.add({ title, type })`; `update(id, …)` turns a `loading` one into its
 * result, and `close(id)` lets it go.
 *
 * They stack one below another, 8px apart, the newest at the bottom. Each rises in as it arrives and
 * sinks out as it goes while the rest close the gap, and a click anywhere on one lets it go early.
 * Snackbars on a timer pause while the stack is hovered.
 *
 * @public
 * @import import { Snackbars } from "@gitbutler/ui-react/Snackbars.tsx";
 */
export const Snackbars: FC<{
	manager: ToastManager<SnackbarData>;
	/** How long one on a timer stays, in milliseconds. */
	timeout?: number;
	/** On the stack, for a host that lifts it with `--snackbars-bottom`. */
	className?: string;
}> = ({ manager, timeout = 5000, className }) => (
	<Toast.Provider toastManager={manager} timeout={timeout}>
		<Stack className={className} />
	</Toast.Provider>
);
