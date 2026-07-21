import { NavigationIndexContext } from "../OutlineNavigationIndexContext.ts";
import { Row } from "../Row.tsx";
import { projectSlice } from "#ui/projects/state.ts";
import { useAppDispatch, useAppSelector } from "#ui/store.ts";
import { operandIdentityKey, type Operand } from "#ui/operands.ts";
import { navigationIndexIncludes } from "#ui/workspace/navigation-index.ts";
import { ComponentProps, FC, use } from "react";
import { assert } from "#ui/assert.ts";

export const ItemRow: FC<
	{
		projectId: string;
		operand: Operand;
	} & Omit<ComponentProps<typeof Row>, "inert" | "isSelected" | "onSelect">
> = ({ projectId, operand, ...props }) => {
	const dispatch = useAppDispatch();
	const navigationIndex = assert(use(NavigationIndexContext));
	const isSelected = useAppSelector((state) =>
		projectSlice.selectors.selectIsSelectedOutline(state, projectId, navigationIndex, operand),
	);
	const selectItem = () => {
		dispatch(projectSlice.actions.selectOutline({ projectId, selection: operand }));
	};

	return (
		<Row
			{...props}
			inert={!navigationIndexIncludes(navigationIndex, operand, operandIdentityKey)}
			isSelected={isSelected}
			onSelect={selectItem}
		/>
	);
};
