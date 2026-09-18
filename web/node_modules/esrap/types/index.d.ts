declare module 'esrap' {
	type BaseNode = {
		type: string;
		loc?: null | {
			start: { line: number; column: number };
			end: { line: number; column: number };
		};
	};

	type NodeOf<T extends string, X> = X extends { type: T } ? X : never;

	type SpecialisedVisitors<T extends BaseNode> = {
		[K in T['type']]?: Visitor<NodeOf<K, T>>;
	};

	type Visitor<T> = (node: T, context: Context) => void;

	export type Visitors<T extends BaseNode = BaseNode> = T['type'] extends '_'
		? never
		: SpecialisedVisitors<T> & { _?: (node: T, context: Context, visit: (node: T) => void) => void };

	interface Location {
		type: 'Location';
		line: number;
		column: number;
	}

	type Command = string | number | Location | Command[];

	export interface PrintOptions {
		sourceMapSource?: string;
		sourceMapContent?: string;
		sourceMapEncodeMappings?: boolean; // default true
		indent?: string; // default tab
	}
	/**
	 * @returns // TODO
	 */
	export function print<T extends BaseNode = BaseNode>(node: T, visitors: Visitors<T>, opts?: PrintOptions): {
		code: string;
		map: any;
	};
	export class Context {
		
		constructor(visitors: Visitors, commands?: Command[]);
		multiline: boolean;
		indent(): void;
		dedent(): void;
		margin(): void;
		newline(): void;
		space(): void;
		
		append(context: Context): void;
		
		write(content: string, node?: BaseNode): void;
		
		location(line: number, column: number): void;
		
		visit(node: {
			type: string;
		}): void;
		empty(): boolean;
		measure(): number;
		"new"(): Context;
		#private;
	}

	export {};
}

declare module 'esrap/languages/ts' {
	import type { Context } from 'esrap';
	export type Node = BaseNode;
	export default function ts(options?: TSOptions): Visitors<BaseNode>;
	// was Record<TSESTree.Expression['type'] | 'Super' | 'RestElement', number>
	export const EXPRESSIONS_PRECEDENCE: Record<string, number>;
	type BaseNode = {
		type: string;
		loc?: null | {
			start: { line: number; column: number };
			end: { line: number; column: number };
		};
	};

	type NodeOf<T extends string, X> = X extends { type: T } ? X : never;

	type SpecialisedVisitors<T extends BaseNode> = {
		[K in T['type']]?: Visitor<NodeOf<K, T>>;
	};

	type Visitor<T> = (node: T, context: Context) => void;

	type Visitors<T extends BaseNode = BaseNode> = T['type'] extends '_'
		? never
		: SpecialisedVisitors<T> & { _?: (node: T, context: Context, visit: (node: T) => void) => void };
	type TSOptions = {
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

	export {};
}

declare module 'esrap/languages/tsx' {
	import type { Context } from 'esrap';
	export type Node = BaseNode;
	export default function tsx(options?: TSOptions): Visitors<BaseNode>;
	type BaseNode = {
		type: string;
		loc?: null | {
			start: { line: number; column: number };
			end: { line: number; column: number };
		};
	};

	type NodeOf<T extends string, X> = X extends { type: T } ? X : never;

	type SpecialisedVisitors<T extends BaseNode> = {
		[K in T['type']]?: Visitor<NodeOf<K, T>>;
	};

	type Visitor<T> = (node: T, context: Context) => void;

	type Visitors<T extends BaseNode = BaseNode> = T['type'] extends '_'
		? never
		: SpecialisedVisitors<T> & { _?: (node: T, context: Context, visit: (node: T) => void) => void };
	type TSOptions = {
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

	export {};
}

//# sourceMappingURL=index.d.ts.map