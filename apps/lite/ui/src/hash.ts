// djb2 algo
export const hash = (str: string): number => {
	let hash = 5381;
	for (let i = 0; i < str.length; i++) hash = (hash * 33) ^ str.charCodeAt(i);

	return hash >>> 0;
};

export const combineHashes = (acc: number, value: number): number => (acc * 53) ^ value;
