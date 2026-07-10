const lastOpenedProjectKey = "lastProject";

export const readLastOpenedProject = (): string | null =>
	window.localStorage.getItem(lastOpenedProjectKey);

export const writeLastOpenedProject = (projectId: string): void =>
	window.localStorage.setItem(lastOpenedProjectKey, projectId);
