import { Dialog } from "@base-ui/react";
import type { FC } from "react";
import styles from "./Settings.module.css";
import { useSuspenseQuery } from "@tanstack/react-query";
import { getGUISettingsQueryOptions, listEditorsQueryOptions } from "#ui/api/queries.ts";
import { useSaveGUISettings } from "#ui/api/mutations.ts";

type Props = {
	open: boolean;
	onOpenChange: (open: boolean) => void;
};

export const Settings: FC<Props> = ({ open, onOpenChange }) => {
	const { data: editors } = useSuspenseQuery(listEditorsQueryOptions);
	const { data: editorId } = useSuspenseQuery({
		...getGUISettingsQueryOptions(),
		select: (cfg) => cfg.editorId,
	});
	const saveGUISettings = useSaveGUISettings();

	return (
		<Dialog.Root open={open} onOpenChange={onOpenChange}>
			<Dialog.Portal>
				<Dialog.Backdrop className={styles.backdrop} />
				<Dialog.Viewport className={styles.viewport}>
					<Dialog.Popup aria-labelledby="settings-heading" className={styles.popup}>
						<h1
							id="settings-heading"
							className="text-15 text-semibold"
							style={{ marginBlockEnd: 16 }}
						>
							Settings
						</h1>

						<label
							htmlFor="editor"
							className="text-12 text-semibold"
							style={{ color: "var(--text-2)" }}
						>
							Default editor
						</label>
						<div className="text-12">
							<select
								id="editor"
								value={editorId ?? ""}
								onChange={(evt) =>
									saveGUISettings.mutate({
										editorId: evt.currentTarget.value,
									})
								}
							>
								<option value="" disabled>
									Select an editor...
								</option>
								{editors.map((editor) => (
									<option key={editor.id} value={editor.id}>
										{editor.name}
									</option>
								))}
							</select>
						</div>
					</Dialog.Popup>
				</Dialog.Viewport>
			</Dialog.Portal>
		</Dialog.Root>
	);
};
