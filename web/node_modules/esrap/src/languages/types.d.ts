import type { BaseNode } from '../types';

export type TSOptions = {
	quotes?: 'double' | 'single';
	/**
	 * Anchor structural tokens (array/object brackets and braces, preserved
	 * parentheses, unary operators, and the closing tokens of calls and
	 * computed member access) with one-character source locations, so
	 * node-boundary positions always resolve through the source map instead of
	 * being attributed to the previous token. Denser maps; identical output.
	 */
	boundaryTokens?: boolean;
	comments?: Comment[];
	getLeadingComments?: (node: BaseNode) => BaseComment[] | undefined;
	getTrailingComments?: (node: BaseNode) => BaseComment[] | undefined;
};

interface Position {
	line: number;
	column: number;
}

// this exists in TSESTree but because of the inanity around enums
// it's easier to do this ourselves
export interface BaseComment {
	type: 'Line' | 'Block';
	value: string;
	start?: number;
	end?: number;
}

export interface Comment extends BaseComment {
	loc: {
		start: Position;
		end: Position;
	};
}
