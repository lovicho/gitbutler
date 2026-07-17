import type { Operand } from "#ui/operands.ts";

export type DragData = {
	sources: Array<Operand>;
};

export const parseDragData = (data: unknown): DragData | null => {
	if (typeof data !== "object" || data === null || !("sources" in data)) return null;
	return data as DragData;
};
