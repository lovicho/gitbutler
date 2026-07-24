import type { SelectionSide } from "@pierre/diffs";

export type DiffLineTarget = {
	itemId: string;
	lineNumber: number;
	side: SelectionSide;
};

const selectionSideFromLineNumber = (element: HTMLElement): SelectionSide | null => {
	switch (element.getAttribute("data-line-type")) {
		case "change-addition":
			return "additions";
		case "change-deletion":
			return "deletions";
		default:
			return null;
	}
};

export const diffLineTargetFromElement = ({
	element,
	itemId,
}: {
	element: HTMLElement;
	itemId: string;
}): DiffLineTarget | null => {
	const side = selectionSideFromLineNumber(element);
	if (side === null) return null;

	const lineNumber = Number.parseInt(element.getAttribute("data-column-number") ?? "", 10);
	if (!Number.isFinite(lineNumber)) return null;

	return {
		itemId,
		lineNumber,
		side,
	};
};
