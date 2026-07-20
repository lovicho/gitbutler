import { getButtonClassName } from "#ui/components/Button.tsx";
import { Icon } from "#ui/components/Icon.tsx";
import { TooltipPopup } from "#ui/components/Tooltip.tsx";
import { projectSlice } from "#ui/projects/state.ts";
import { useAppDispatch, useAppSelector } from "#ui/store.ts";
import { workspaceHotkeys } from "#ui/hotkeys.ts";
import { Tooltip } from "@base-ui/react";
import { useParams } from "@tanstack/react-router";
import { type FC } from "react";
import styles from "./TopLeftControls.module.css";

const FullWindowButton: FC = () => {
	const { id: projectId } = useParams({ from: "/project/$id/workspace" });
	const dispatch = useAppDispatch();
	const fullWindow = useAppSelector((state) =>
		projectSlice.selectors.selectDetailsFullWindow(state, projectId),
	);

	return (
		<Tooltip.Root>
			<Tooltip.Trigger
				render={
					<button
						type="button"
						className={getButtonClassName({ iconOnly: true, variant: "ghost" })}
						aria-label={workspaceHotkeys.toggleOutline.meta.name}
						onClick={() =>
							dispatch(
								projectSlice.actions.setDetailsFullWindow({ projectId, fullWindow: !fullWindow }),
							)
						}
					>
						{fullWindow ? <Icon name="sidebar-show" /> : <Icon name="sidebar-hide" />}
					</button>
				}
			/>
			<Tooltip.Portal>
				<Tooltip.Positioner sideOffset={4}>
					<Tooltip.Popup render={<TooltipPopup kbd={workspaceHotkeys.toggleOutline.hotkey} />}>
						{workspaceHotkeys.toggleOutline.meta.name}
					</Tooltip.Popup>
				</Tooltip.Positioner>
			</Tooltip.Portal>
		</Tooltip.Root>
	);
};

const isMac = window.lite.platform === "darwin";

export const TopLeftControls: FC = () => (
	<div className={styles.container}>
		{isMac && <div className={styles.macSpacer} />}
		<FullWindowButton />
	</div>
);
