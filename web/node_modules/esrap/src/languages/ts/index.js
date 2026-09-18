/** @import { TSESTree } from '@typescript-eslint/types' */
/** @import { Visitors } from '../../types.js' */
/** @import { TSOptions, BaseComment } from '../types.js' */
import { Context } from 'esrap';

/** @typedef {TSESTree.Node} Node */

/** @type {Record<TSESTree.Expression['type'] | 'Super' | 'RestElement', number>} */
export const EXPRESSIONS_PRECEDENCE = {
	JSXFragment: 20,
	JSXElement: 20,
	ArrayPattern: 20,
	ObjectPattern: 20,
	ArrayExpression: 20,
	TaggedTemplateExpression: 20,
	ThisExpression: 20,
	Identifier: 20,
	TemplateLiteral: 20,
	Super: 20,
	SequenceExpression: 20,
	MemberExpression: 19,
	MetaProperty: 19,
	CallExpression: 19,
	ChainExpression: 19,
	ImportExpression: 19,
	NewExpression: 19,
	Literal: 18,
	TSInstantiationExpression: 18,
	TSNonNullExpression: 18,
	TSTypeAssertion: 18,
	AwaitExpression: 17,
	ClassExpression: 17,
	FunctionExpression: 17,
	ObjectExpression: 17,
	UpdateExpression: 16,
	UnaryExpression: 15,
	BinaryExpression: 14,
	// `as`/`satisfies` sit between binary and logical operators
	TSAsExpression: 13,
	TSSatisfiesExpression: 13,
	LogicalExpression: 12,
	ConditionalExpression: 4,
	ArrowFunctionExpression: 3,
	AssignmentExpression: 3,
	YieldExpression: 2,
	RestElement: 1
};

const OPERATOR_PRECEDENCE = {
	'||': 2,
	'&&': 3,
	'??': 4,
	'|': 5,
	'^': 6,
	'&': 7,
	'==': 8,
	'!=': 8,
	'===': 8,
	'!==': 8,
	'<': 9,
	'>': 9,
	'<=': 9,
	'>=': 9,
	in: 9,
	instanceof: 9,
	'<<': 10,
	'>>': 10,
	'>>>': 10,
	'+': 11,
	'-': 11,
	'*': 12,
	'%': 12,
	'/': 12,
	'**': 13
};

/**
 * Writes `keyword` bounded by source map locations for the exact character span,
 * so breakpoints line up on keywords (not only identifiers and braces).
 *
 * @param {import('esrap').Context} context
 * @param {number} line ESTree / acorn 1-based line
 * @param {number} column 0-based ESTree column (UTF-16 indices, same as emitted JS)
 * @param {string} keyword
 */
function write_source_keyword(context, line, column, keyword) {
	context.location(line, column);
	context.write(keyword);
	context.location(line, column + keyword.length);
}

/**
 * Sequential fragments from one ESTree `loc.start`, advancing columns for source mappings.
 * Pass each printed fragment exactly (usually including trailing spaces), e.g. `declare `, `class `.
 * Usage: `const kw = create_keyword_write(context, node, predicate);`
 *
 * @param {import('esrap').Context} context
 * @param {TSESTree.Node} node
 * @param {(n: any) => boolean} [map_ok] When false or missing `loc`, writes go through `context.write` only.
 * @returns {(fragment: string) => void}
 */
function create_keyword_write(context, node, map_ok) {
	let cursor =
		node.loc && (!map_ok || map_ok(node))
			? { line: node.loc.start.line, col: node.loc.start.column }
			: null;

	return (fragment) => {
		if (cursor) {
			write_source_keyword(context, cursor.line, cursor.col, fragment);
			cursor.col += fragment.length;
		} else {
			context.write(fragment);
		}
	};
}

/**
 * Map one keyword at `node.loc.start`, then append unmapped `suffix` (spaces, punctuation, etc.).
 *
 * @param {import('esrap').Context} context
 * @param {TSESTree.Node} node
 * @param {string} keyword
 * @param {string} [suffix='']
 */
function write_keyword(context, node, keyword, suffix = '') {
	if (node.loc) {
		write_source_keyword(context, node.loc.start.line, node.loc.start.column, keyword);
		context.write(suffix);
	} else {
		context.write(keyword + suffix);
	}
}

/**
 * `async function` offset heuristics assume the `function` token is on the same line as `async`
 * and anchored with the function `id`/`body` ESTree locations.
 *
 * @param {TSESTree.FunctionDeclaration | TSESTree.FunctionExpression} node
 */
function function_async_function_offset_ok(node) {
	const line = /** @type {{ loc?: { start: { line: number } } }} */ (node).loc?.start.line;
	if (line === undefined) return false;

	return node.id?.loc?.start.line === line || node.body?.loc?.start.line === line;
}

/** @param {{ loc?: { start: { line: number }, end: { line: number } } }} node */
function single_line_node(node) {
	return !!(node.loc && node.loc.start.line === node.loc.end.line);
}

/**
 * @param {TSESTree.ClassDeclaration | TSESTree.ClassExpression} node
 * @returns {boolean}
 */
function class_modifier_keywords_map_ok(node) {
	return (
		!!node.loc &&
		!node.decorators?.length &&
		(node.id
			? node.id.loc?.start.line === node.loc.start.line
			: node.body.loc.start.line === node.loc.start.line)
	);
}

/**
 * @param {TSESTree.MethodDefinition | TSESTree.TSAbstractMethodDefinition} node
 * @returns {boolean}
 */
function method_modifiers_keywords_map_ok(node) {
	return !!(
		node.loc &&
		!node.decorators?.length &&
		node.loc.start.line === node.value.loc?.start.line
	);
}

/**
 * @param {TSESTree.PropertyDefinition | TSESTree.TSAbstractPropertyDefinition | TSESTree.AccessorProperty | TSESTree.TSAbstractAccessorProperty} node
 * @returns {boolean}
 */
function field_modifiers_keywords_map_ok(node) {
	if (!node.loc || node.decorators?.length) return false;
	if (!node.value?.loc) return true;
	return node.loc.start.line === node.value.loc.start.line;
}

/**
 * @param {BaseComment} comment
 * @param {Context} context
 */
function write_comment(comment, context) {
	if (comment.type === 'Line') {
		context.write(`//${comment.value}`);
	} else {
		context.write('/*');
		const lines = comment.value.split('\n');

		for (let i = 0; i < lines.length; i += 1) {
			if (i > 0) context.newline();
			context.write(lines[i]);
		}

		context.write('*/');
		if (lines.length > 1) context.newline();
	}
}

/**
 * @param {TSOptions} [options]
 * @returns {Visitors<TSESTree.Node>}
 */
export default (options = {}) => {
	const quote_char = options.quotes === 'double' ? '"' : "'";

	const comments = options.comments ?? [];

	let comment_index = 0;

	/**
	 * Write additional comments for a node
	 * @param {Context} context
	 * @param {BaseComment[] | undefined} comments
	 * @param {('leading' | 'trailing')} position
	 */
	function write_additional_comments(context, comments, position) {
		if (!comments) {
			return;
		}

		for (let i = 0; i < comments.length; i += 1) {
			const comment = comments[i];

			if (position === 'trailing' && i === 0) {
				context.write(' ');
			}

			write_comment(comment, context);

			if (position === 'leading') {
				if (comment.type === 'Line') {
					context.newline();
				} else if (comment.type === 'Block' && !comment.value.includes('\n')) {
					context.write(' ');
				}
			}
		}
	}

	/**
	 * Set `comment_index` to be the first comment after `start`.
	 * Most of the time this is already correct, but if nodes
	 * have been moved around we may need to search for it
	 * @param {TSESTree.Node} node
	 */
	function reset_comment_index(node) {
		if (!node.loc) {
			comment_index = comments.length;
			return;
		}

		let previous = comments[comment_index - 1];
		let comment = comments[comment_index];

		if (
			comment &&
			comment.loc &&
			!before(comment.loc.start, node.loc.start) &&
			(!previous || (previous.loc && before(previous.loc.start, node.loc.start)))
		) {
			return;
		}

		// TODO use a binary search here, account for synthetic nodes (without `loc`)
		comment_index = comments.findIndex(
			(comment) => comment.loc && node.loc && !before(comment.loc.start, node.loc.start)
		);
		if (comment_index === -1) comment_index = comments.length;
	}

	/**
	 * @param {Context} context
	 * @param {{ line: number, column: number } | null} prev
	 * @param {{ line: number, column: number } | null} next
	 */
	function flush_trailing_comments(context, prev, next) {
		while (comment_index < comments.length) {
			const comment = comments[comment_index];

			if (
				comment &&
				prev &&
				comment.loc.start.line === prev.line &&
				(next === null || before(comment.loc.end, next))
			) {
				context.write(' ');
				write_comment(comment, context);

				comment_index += 1;

				if (comment.type === 'Line') {
					context.newline();
				} else {
					continue;
				}
			}

			break;
		}
	}

	/**
	 * @param {Context} context
	 * @param {{ line: number, column: number } | null} from
	 * @param {{ line: number, column: number }} to
	 * @param {boolean} pad
	 */
	function flush_comments_until(context, from, to, pad) {
		let first = true;

		while (comment_index < comments.length) {
			const comment = comments[comment_index];

			if (comment && comment.loc && to && before(comment.loc.start, to)) {
				if (first && from !== null && comment.loc.start.line > from.line) {
					context.margin();
					context.newline();
				}

				first = false;

				write_comment(comment, context);

				if (comment.loc.end.line < to.line) {
					context.newline();
				} else if (pad) {
					context.write(' ');
				}

				comment_index += 1;
			} else {
				break;
			}
		}
	}

	/**
	 * @param {TSESTree.Node} node
	 * @returns {boolean}
	 */
	function has_object_or_array_value(node) {
		if (!node || node.type !== 'Property') return false;
		const value = node.value?.type === 'AssignmentPattern' ? node.value.left : node.value;
		return value?.type === 'ObjectExpression' || value?.type === 'ArrayExpression';
	}

	/**
	 * @param {Context} context
	 * @param {TSESTree.Node[]} nodes
	 * @param {{ line: number, column: number }} until
	 * @param {boolean} pad
	 */
	function sequence(context, nodes, until, pad, separator = ',', trailing_newline = true) {
		let multiline = false;
		let length = -1;

		/** @type {boolean[]} */
		const multiline_nodes = [];

		const children = nodes.map((child, i) => {
			const child_context = context.new();
			if (child) child_context.visit(child);

			multiline_nodes[i] = child_context.multiline;

			if (i < nodes.length - 1 || !child) {
				child_context.write(separator);
			}

			const next = i === nodes.length - 1 ? until : nodes[i + 1]?.loc?.start || null;
			flush_trailing_comments(child_context, child?.loc?.end || null, next);

			length += child_context.measure() + 1;
			multiline ||= child_context.multiline;

			return child_context;
		});

		multiline ||= length > 60;

		if (multiline) {
			context.indent();
			context.newline();
		} else if (pad && length > 0) {
			context.write(' ');
		}

		/** @type {Context | null} */
		let prev = null;

		for (let i = 0; i < nodes.length; i += 1) {
			const child = children[i];

			if (prev !== null) {
				if (multiline_nodes[i - 1] && multiline_nodes[i]) {
					if (!has_object_or_array_value(nodes[i - 1]) || !has_object_or_array_value(nodes[i])) {
						context.margin();
					}
				}

				if (nodes[i]) {
					if (multiline) {
						context.newline();
					} else {
						context.write(' ');
					}
				}
			}

			context.append(child);

			prev = child;
		}

		flush_comments_until(context, nodes[nodes.length - 1]?.loc?.end ?? null, until, false);

		if (multiline) {
			context.dedent();
			if (trailing_newline) context.newline();
		} else if (pad && length > 0) {
			context.write(' ');
		}
	}

	/**
	 * Push a sequence of nodes onto separate lines, separating them with
	 * an extra newline where appropriate
	 * @param {Context} context
	 * @param {TSESTree.Node & { body: TSESTree.Node[] }} node
	 */
	function body(context, node) {
		reset_comment_index(node);

		/** @type {string | null} */
		let prev_type = null;
		let prev_multiline = false;

		for (let i = 0; i < node.body.length; i += 1) {
			const child = node.body[i];
			if (child.type === 'EmptyStatement') continue;

			const child_context = context.new();
			child_context.visit(child);

			if (prev_type !== null) {
				if (child_context.multiline || prev_multiline || child.type !== prev_type) {
					context.margin();
				}

				context.newline();
			}

			context.append(child_context);

			flush_trailing_comments(
				context,
				child.loc?.end || null,
				node.body[i + 1]?.loc?.end ?? node.loc?.end ?? null
			);

			prev_type = child.type;
			prev_multiline = child_context.multiline;
		}

		if (node.loc) {
			context.newline();
			flush_comments_until(
				context,
				node.body[node.body.length - 1]?.loc?.end ?? null,
				node.loc.end,
				false
			);
		}
	}

	const boundary_tokens = options.boundaryTokens === true;

	/**
	 * A one-character synthetic token node so structural tokens (`(`, `[`, `{`,
	 * unary operators, …) written where a node's SOURCE span begins or ends get a
	 * sourcemap anchor. Without it, everything up to the next mapped token is
	 * attributed to the PREVIOUS token's source position — `write(content, node)`
	 * only maps tokens written with a node, and these boundary characters belong
	 * to no written token. Opt-in (`boundaryTokens`): denser maps, byte-identical
	 * output.
	 * @param {{ line: number, column: number } | undefined} pos
	 * @param {number} [length]
	 */
	function token_at(pos, length = 1) {
		if (!boundary_tokens) return undefined;
		if (!pos) return undefined;
		return /** @type {any} */ ({
			loc: { start: pos, end: { line: pos.line, column: pos.column + length } }
		});
	}

	/** @param {{ line: number, column: number } | undefined} pos */
	function token_before(pos) {
		if (!boundary_tokens) return undefined;
		if (!pos || pos.column === 0) return undefined;
		return /** @type {any} */ ({
			loc: { start: { line: pos.line, column: pos.column - 1 }, end: pos }
		});
	}

	const shared = {
		/**
		 * @param {TSESTree.ArrayExpression | TSESTree.ArrayPattern} node
		 * @param {Context} context
		 */
		'ArrayExpression|ArrayPattern': (node, context) => {
			context.write('[', token_at(node.loc?.start));
			sequence(
				context,
				/** @type {TSESTree.Node[]} */ (node.elements),
				node.loc?.end ?? null,
				false
			);
			context.write(']', token_before(node.loc?.end));
		},

		/**
		 * @param {TSESTree.BinaryExpression | TSESTree.LogicalExpression} node
		 * @param {Context} context
		 */
		'BinaryExpression|LogicalExpression': (node, context) => {
			// TODO
			// const is_in = node.operator === 'in';
			// if (is_in) {
			// 	// Avoids confusion in `for` loops initializers
			// 	chunks.write('(');
			// }
			maybe_wrap(context, node.left, operand_needs_wrap(node.left, node, false));

			context.write(` ${node.operator} `);

			maybe_wrap(context, node.right, operand_needs_wrap(node.right, node, true));
		},

		/**
		 * @param {TSESTree.BlockStatement | TSESTree.ClassBody} node
		 * @param {Context} context
		 */
		'BlockStatement|ClassBody': (node, context) => {
			if (node.loc) {
				const { line, column } = node.loc.start;
				context.location(line, column);
				context.write('{');
				context.location(line, column + 1);
			} else {
				context.write('{');
			}

			const child_context = context.new();
			body(child_context, node);

			if (!child_context.empty()) {
				context.indent();
				context.newline();
				context.append(child_context);
				context.dedent();
				context.newline();
			}

			if (node.loc) {
				const { line, column } = node.loc.end;

				context.location(line, column - 1);
				context.write('}');
				context.location(line, column);
			} else {
				context.write('}');
			}
		},

		/**
		 * @param {TSESTree.CallExpression | TSESTree.NewExpression} node
		 * @param {Context} context
		 */
		'CallExpression|NewExpression': (node, context) => {
			if (node.type === 'NewExpression') {
				write_keyword(context, node, 'new', ' ');
			}

			const wrap =
				node.callee.type === 'ChainExpression' ||
				EXPRESSIONS_PRECEDENCE[node.callee.type] < EXPRESSIONS_PRECEDENCE.CallExpression ||
				(node.type === 'NewExpression' && has_call_expression(node.callee));
			maybe_wrap(context, node.callee, wrap);

			if (/** @type {TSESTree.CallExpression} */ (node).optional) {
				context.write('?.');
			}

			if (node.typeArguments) context.visit(node.typeArguments);

			const open = context.new();
			const join = context.new();

			context.write('(');
			context.append(open);

			// if the final argument is multiline, it doesn't need to force all the
			// other arguments to also be multiline
			const child_context = context.new();
			const final_context = context.new();

			context.append(child_context);
			context.append(final_context);

			for (let i = 0; i < node.arguments.length; i += 1) {
				const is_last = i === node.arguments.length - 1;
				const context = is_last ? final_context : child_context;
				const arg = node.arguments[i];

				// special case — if final argument has a comment above it,
				// we make the whole sequence multiline
				if (
					is_last &&
					arg.loc &&
					comments[comment_index] &&
					comments[comment_index].loc &&
					comments[comment_index].loc.start.line < arg.loc.start.line
				) {
					child_context.multiline = true;
				}

				context.visit(arg);

				if (!is_last) context.write(',');

				const next = is_last
					? (node.loc?.end ?? null)
					: (node.arguments[i + 1]?.loc?.start ?? null);

				flush_trailing_comments(context, arg.loc?.end ?? null, next);

				if (!is_last) context.append(join);
			}

			context.multiline ||= child_context.multiline || final_context.multiline;

			if (child_context.multiline) {
				open.indent();
				open.newline();
				join.newline();
				context.dedent();
				context.newline();
			} else {
				join.write(' ');
			}

			context.write(')', token_before(node.loc?.end));
		},

		/**
		 * @param {TSESTree.ClassDeclaration | TSESTree.ClassExpression} node
		 * @param {Context} context
		 */
		'ClassDeclaration|ClassExpression': (node, context) => {
			if (node.decorators) {
				for (const decorator of node.decorators) {
					context.visit(decorator);
				}
			}

			const kw = create_keyword_write(context, node, class_modifier_keywords_map_ok);

			if (node.declare) kw('declare ');
			if (node.abstract) kw('abstract ');
			kw('class ');

			if (node.id) {
				context.visit(node.id);
			}

			if (node.typeParameters) {
				context.visit(node.typeParameters);
			}

			if (node.id || node.typeParameters) {
				context.write(' ');
			}

			if (node.superClass) {
				context.write('extends ');
				// the `extends` clause is a LeftHandSideExpression; anything lower (a
				// logical/binary/conditional/etc.) must be parenthesized
				const wrap_super =
					EXPRESSIONS_PRECEDENCE[node.superClass.type] < EXPRESSIONS_PRECEDENCE.NewExpression;
				if (wrap_super) context.write('(');
				context.visit(node.superClass);
				if (wrap_super) context.write(')');

				// @ts-expect-error `acorn-typescript` and `@typescript-eslint/types` have slightly different type definitions
				var type_arguments = node.superTypeParameters ?? node.superTypeArguments;
				if (type_arguments) {
					context.visit(type_arguments);
				}

				context.write(' ');
			}

			if (node.implements && node.implements.length > 0) {
				context.write('implements');
				sequence(context, node.implements, node.body.loc?.start ?? null, true);
			}

			context.visit(node.body);
		},

		/**
		 * @param {TSESTree.ForInStatement | TSESTree.ForOfStatement} node
		 * @param {Context} context
		 */
		'ForInStatement|ForOfStatement': (node, context) => {
			const kw = create_keyword_write(context, node);

			kw('for ');
			if (node.type === 'ForOfStatement' && node.await) {
				if (single_line_node(node)) kw('await ');
				else context.write('await ');
			}

			context.write('(');

			if (node.left.type === 'VariableDeclaration') {
				handle_var_declaration(node.left, context);
			} else {
				context.visit(node.left);
			}

			context.write(node.type === 'ForInStatement' ? ' in ' : ' of ');
			context.visit(node.right);
			context.write(') ');
			context.visit(node.body);
		},

		/**
		 * @param {TSESTree.FunctionDeclaration | TSESTree.FunctionExpression} node
		 * @param {Context} context
		 */
		'FunctionDeclaration|FunctionExpression': (node, context) => {
			if (!node.loc?.start) {
				if (node.async) context.write('async ');
				context.write(node.generator ? 'function* ' : 'function ');
			} else {
				const { line, column } = node.loc.start;

				if (node.async) {
					if (function_async_function_offset_ok(node)) {
						write_source_keyword(context, line, column, 'async ');
						const col2 = column + 'async '.length;
						write_source_keyword(context, line, col2, 'function');
						context.write(node.generator ? '* ' : ' ');
					} else {
						context.write('async ');
						context.write(node.generator ? 'function* ' : 'function ');
					}
				} else if (node.generator) {
					write_source_keyword(context, line, column, 'function');
					context.write('* ');
				} else {
					write_source_keyword(context, line, column, 'function');
					context.write(' ');
				}
			}

			if (node.id) context.visit(node.id);

			if (node.typeParameters) {
				context.visit(node.typeParameters);
			}

			context.write('(');
			sequence(context, node.params, (node.returnType ?? node.body).loc?.start ?? null, false);
			context.write(')');

			if (node.returnType) context.visit(node.returnType);

			context.write(' ');

			context.visit(node.body);
		},

		/**
		 * @param {TSESTree.MethodDefinition | TSESTree.TSAbstractMethodDefinition} node
		 * @param {Context} context
		 */
		'MethodDefinition|TSAbstractMethodDefinition': (node, context) => {
			if (node.decorators) {
				for (const decorator of node.decorators) {
					context.visit(decorator);
				}
			}

			const kw = create_keyword_write(context, node, method_modifiers_keywords_map_ok);

			// @ts-expect-error `acorn-typescript` and `@typescript-eslint/types` have slightly different type definitions
			if (node.abstract || node.type === 'TSAbstractMethodDefinition') {
				kw('abstract ');
			}

			if (node.accessibility) {
				kw(node.accessibility + ' ');
			}

			if (node.override) {
				kw('override ');
			}

			if (node.static) {
				kw('static ');
			}

			if (node.kind === 'get' || node.kind === 'set') {
				kw(node.kind + ' ');
			}

			if (node.value.async) {
				kw('async ');
			}

			if (node.value.generator) context.write('*');

			if (node.computed) context.write('[', token_before(node.key.loc?.start));
			context.visit(node.key);
			if (node.computed) context.write(']', token_at(node.key.loc?.end));

			// optional method (`m?()`)
			if (node.optional) context.write('?');

			// @ts-expect-error `typeParameters` lives on the method node, not its value
			if (node.typeParameters) context.visit(node.typeParameters);

			context.write('(');
			sequence(
				context,
				node.value.params,
				(node.value.returnType ?? node.value.body)?.loc?.start ?? node.loc?.end ?? null,
				false
			);
			context.write(')');

			if (node.value.returnType) context.visit(node.value.returnType);

			context.write(' ');

			if (node.value.body) context.visit(node.value.body);
		},

		/**
		 * @param {TSESTree.PropertyDefinition | TSESTree.TSAbstractPropertyDefinition | TSESTree.AccessorProperty | TSESTree.TSAbstractAccessorProperty} node
		 * @param {Context} context
		 */
		'PropertyDefinition|TSAbstractPropertyDefinition|AccessorProperty|TSAbstractAccessorProperty': (
			node,
			context
		) => {
			if (node.decorators) {
				for (const decorator of node.decorators) {
					context.visit(decorator);
				}
			}

			const kw = create_keyword_write(context, node, field_modifiers_keywords_map_ok);

			if (node.declare) kw('declare ');

			if (node.accessibility) {
				kw(node.accessibility + ' ');
			}

			if (
				// @ts-expect-error `acorn-typescript` and `@typescript-eslint/types` have slightly different type definitions
				node.abstract ||
				node.type === 'TSAbstractPropertyDefinition' ||
				node.type === 'TSAbstractAccessorProperty'
			) {
				kw('abstract ');
			}

			if (node.static) {
				kw('static ');
			}

			if (node.override) kw('override ');

			if (node.readonly) kw('readonly ');

			if (
				// @ts-expect-error `acorn-typescript` and `@typescript-eslint/types` have slightly different type definitions
				node.accessor ||
				node.type === 'AccessorProperty' ||
				node.type === 'TSAbstractAccessorProperty'
			) {
				kw('accessor ');
			}

			if (node.computed) {
				context.write('[', token_before(node.key.loc?.start));
				context.visit(node.key);
				context.write(']', token_at(node.key.loc?.end));
			} else {
				context.visit(node.key);
			}

			// `x?: T` (optional) / `x!: T` (definite assignment)
			if (node.optional) context.write('?');
			else if (node.definite) context.write('!');

			if (node.typeAnnotation) {
				if (node.type === 'AccessorProperty' || node.type === 'TSAbstractAccessorProperty') {
					context.visit(node.typeAnnotation);
				} else {
					context.write(': ');
					context.visit(node.typeAnnotation.typeAnnotation);
				}
			}

			if (node.value) {
				context.write(' = ');
				context.visit(node.value);
			}

			context.write(';');

			flush_trailing_comments(
				context,
				(node.value ?? node.typeAnnotation ?? node.key).loc?.end ?? null,
				null
			);
		},

		/**
		 * @param {TSESTree.RestElement | TSESTree.SpreadElement} node
		 * @param {Context} context
		 */
		'RestElement|SpreadElement': (node, context) => {
			context.write('...');
			context.visit(node.argument);

			// @ts-expect-error `acorn-typescript` and `@typescript-eslint/types` have slightly different type definitions
			if (node.typeAnnotation) context.visit(node.typeAnnotation);
		},

		/**
		 * @param {TSESTree.TSConstructSignatureDeclaration | TSESTree.TSCallSignatureDeclaration} node
		 * @param {Context} context
		 */
		'TSConstructSignatureDeclaration|TSCallSignatureDeclaration': (node, context) => {
			if (node.type === 'TSConstructSignatureDeclaration') context.write('new');

			if (node.typeParameters) {
				context.visit(node.typeParameters);
			}

			context.write('(');
			sequence(
				context,
				// @ts-expect-error `acorn-typescript` and `@typescript-eslint/types` have slightly different type definitions
				node.parameters ?? node.params,
				// @ts-expect-error `acorn-typescript` and `@typescript-eslint/types` have slightly different type definitions
				(node.typeAnnotation ?? node.returnType)?.loc?.start ?? null,
				false
			);
			context.write(')');

			// @ts-expect-error `acorn-typescript` and `@typescript-eslint/types` have slightly different type definitions
			if (node.typeAnnotation || node.returnType) {
				// @ts-expect-error `acorn-typescript` and `@typescript-eslint/types` have slightly different type definitions
				context.visit(node.typeAnnotation ?? node.returnType);
			}
		},

		/**
		 * @param {TSESTree.TSFunctionType | TSESTree.TSConstructorType} node
		 * @param {Context} context
		 */
		'TSFunctionType|TSConstructorType': (node, context) => {
			if (node.type === 'TSConstructorType') context.write('new ');
			if (node.typeParameters) context.visit(node.typeParameters);

			context.write('(');
			sequence(
				context,
				// @ts-expect-error `acorn-typescript` and `@typescript-eslint/types` have slightly different type definitions
				node.parameters ?? node.params,
				// @ts-expect-error `acorn-typescript` and `@typescript-eslint/types` have slightly different type definitions
				node.typeAnnotation?.typeAnnotation?.loc?.start ??
					node.returnType?.typeAnnotation?.loc?.start ??
					null,
				false
			);
			context.write(')');

			context.write(' => ');

			// @ts-expect-error `acorn-typescript` and `@typescript-eslint/types` have slightly different type definitions
			context.visit(node.typeAnnotation?.typeAnnotation ?? node.returnType?.typeAnnotation);
		}
	};

	return {
		_(node, context, visit) {
			write_additional_comments(context, options.getLeadingComments?.(node), 'leading');

			if (node.loc) {
				flush_comments_until(context, null, node.loc.start, true);
			}

			visit(node);

			write_additional_comments(context, options.getTrailingComments?.(node), 'trailing');
		},

		AccessorProperty:
			shared[
				'PropertyDefinition|TSAbstractPropertyDefinition|AccessorProperty|TSAbstractAccessorProperty'
			],

		ArrayExpression: shared['ArrayExpression|ArrayPattern'],

		ArrayPattern: shared['ArrayExpression|ArrayPattern'],

		ArrowFunctionExpression: (node, context) => {
			if (node.async) {
				write_keyword(context, node, 'async', ' ');
			}

			if (node.typeParameters) {
				context.visit(node.typeParameters);
			}

			context.write('(');
			sequence(context, node.params, (node.returnType ?? node.body).loc?.start ?? null, false);
			context.write(')');

			if (node.returnType) context.visit(node.returnType);

			context.write(' => ');

			maybe_wrap(context, node.body, arrow_concise_body_needs_wrap(node.body));
		},

		AssignmentExpression(node, context) {
			context.visit(node.left);
			context.write(` ${node.operator} `);
			context.visit(node.right);
		},

		AssignmentPattern(node, context) {
			context.visit(node.left);
			context.write(' = ');
			context.visit(node.right);
		},

		AwaitExpression(node, context) {
			if (node.argument) {
				const precedence = EXPRESSIONS_PRECEDENCE[node.argument.type];

				if (precedence && precedence < EXPRESSIONS_PRECEDENCE.AwaitExpression) {
					write_keyword(context, node, 'await', ' (');
					context.visit(node.argument);
					context.write(')');
				} else {
					write_keyword(context, node, 'await', ' ');
					context.visit(node.argument);
				}
			} else {
				write_keyword(context, node, 'await', '');
			}
		},

		BinaryExpression: shared['BinaryExpression|LogicalExpression'],

		BlockStatement: shared['BlockStatement|ClassBody'],

		BreakStatement(node, context) {
			if (node.label) {
				write_keyword(context, node, 'break', ' ');
				context.visit(node.label);
				context.write(';');
			} else {
				write_keyword(context, node, 'break', ';');
			}
		},

		CallExpression: shared['CallExpression|NewExpression'],

		ChainExpression(node, context) {
			context.visit(node.expression);
		},

		ClassBody: shared['BlockStatement|ClassBody'],

		ClassDeclaration: shared['ClassDeclaration|ClassExpression'],

		ClassExpression: shared['ClassDeclaration|ClassExpression'],

		ConditionalExpression(node, context) {
			const wrap =
				EXPRESSIONS_PRECEDENCE[node.test.type] <= EXPRESSIONS_PRECEDENCE.ConditionalExpression;
			maybe_wrap(context, node.test, wrap);

			const consequent = context.new();
			const alternate = context.new();

			// TODO flush comments here, rather than in visitors

			consequent.visit(node.consequent);
			alternate.visit(node.alternate);

			if (
				consequent.multiline ||
				alternate.multiline ||
				consequent.measure() + alternate.measure() > 50
			) {
				context.indent();
				context.newline();
				context.write('? ');
				context.append(consequent);
				context.newline();
				context.write(': ');
				context.append(alternate);
				context.dedent();
			} else {
				context.write(' ? ');
				context.append(consequent);
				context.write(' : ');
				context.append(alternate);
			}
		},

		ContinueStatement(node, context) {
			if (node.label) {
				write_keyword(context, node, 'continue', ' ');
				context.visit(node.label);
				context.write(';');
			} else {
				write_keyword(context, node, 'continue', ';');
			}
		},

		DebuggerStatement(node, context) {
			context.write('debugger', node);
			context.write(';');
		},

		Decorator(node, context) {
			context.write('@');
			// a decorator must be an identifier/member/call (or parenthesized); anything
			// else (ternary, logical, assignment, unary, `as`, optional chain…) needs wrapping
			const wrap =
				/** @type {string} */ (node.expression.type) === 'ChainExpression' ||
				EXPRESSIONS_PRECEDENCE[node.expression.type] < EXPRESSIONS_PRECEDENCE.CallExpression;
			if (wrap) context.write('(');
			context.visit(node.expression);
			if (wrap) context.write(')');
			context.newline();
		},

		DoWhileStatement(node, context) {
			write_keyword(context, node, 'do', ' ');
			context.visit(node.body);

			const test_loc = node.test.loc?.start;
			const body_end = node.body.loc?.end;
			if (test_loc && body_end && body_end.line === test_loc.line && test_loc.column >= 6) {
				context.write(' ');
				write_source_keyword(context, body_end.line, body_end.column + 1, 'while');
				context.write(' (');
			} else {
				context.write(' while (');
			}

			context.visit(node.test);
			context.write(');');
		},

		EmptyStatement(node, context) {
			context.write(';');
		},

		ExportAllDeclaration(node, context) {
			context.write(node.exportKind === 'type' ? 'export type * ' : 'export * ');

			if (node.exported) {
				context.write('as ');
				context.visit(node.exported);
			}

			context.write(' from ');
			context.visit(node.source);
			context.write(';');
		},

		ExportDefaultDeclaration(node, context) {
			const kw = create_keyword_write(context, node, single_line_node);
			kw('export ');
			kw('default ');

			context.visit(node.declaration);

			if (node.declaration.type !== 'FunctionDeclaration') {
				context.write(';');
			}
		},

		ExportNamedDeclaration(node, context) {
			if (node.declaration) {
				// Check if declaration has decorators (ClassDeclaration, ClassExpression can have them)
				const decl = /** @type {any} */ (node.declaration);
				if (decl.decorators && decl.decorators.length > 0) {
					for (const decorator of decl.decorators) {
						context.visit(decorator);
					}
					write_keyword(context, node, 'export', ' ');
					// Temporarily remove decorators so ClassDeclaration doesn't print them again
					const savedDecorators = decl.decorators;
					decl.decorators = [];
					context.visit(node.declaration);
					decl.decorators = savedDecorators;
				} else {
					write_keyword(context, node, 'export', ' ');
					context.visit(node.declaration);
				}
				return;
			}

			const kw = create_keyword_write(context, node);

			kw('export ');
			if (node.exportKind === 'type') {
				if (single_line_node(node)) kw('type ');
				else context.write('type ');
			}

			context.write('{');
			sequence(context, node.specifiers, node.source?.loc?.start ?? node.loc?.end ?? null, true);
			context.write('}');

			if (node.source) {
				context.write(' from ');
				context.visit(node.source);
			}

			context.write(';');
		},

		ExportSpecifier(node, context) {
			if (node.exportKind === 'type') {
				context.write('type ');
			}

			context.visit(node.local);

			if (
				node.local.type === 'Identifier' &&
				node.exported.type === 'Identifier' &&
				node.local.name !== node.exported.name
			) {
				context.write(' as ');
				context.visit(node.exported);
			}
		},

		ExpressionStatement(node, context) {
			// would otherwise be parsed as a block / function / class declaration
			const wrap = leads_with_curly_or_keyword(node.expression);
			maybe_wrap(context, node.expression, wrap);
			context.write(';');
		},

		ForStatement: (node, context) => {
			write_keyword(context, node, 'for', ' (');

			if (node.init) {
				if (node.init.type === 'VariableDeclaration') {
					handle_var_declaration(node.init, context);
				} else {
					context.visit(node.init);
				}
			}

			context.write('; ');
			if (node.test) context.visit(node.test);
			context.write('; ');
			if (node.update) context.visit(node.update);

			context.write(') ');
			context.visit(node.body);
		},

		ForInStatement: shared['ForInStatement|ForOfStatement'],

		ForOfStatement: shared['ForInStatement|ForOfStatement'],

		FunctionDeclaration: shared['FunctionDeclaration|FunctionExpression'],

		FunctionExpression: shared['FunctionDeclaration|FunctionExpression'],

		Identifier(node, context) {
			let name = node.name;
			context.write(name, node);

			// optional parameters (`a?: T`) carry `optional` on the identifier
			if (node.optional) context.write('?');

			if (node.typeAnnotation) context.visit(node.typeAnnotation);
		},

		IfStatement(node, context) {
			write_keyword(context, node, 'if', ' (');
			context.visit(node.test);
			context.write(') ');
			context.visit(node.consequent);

			if (node.alternate) {
				context.space();

				const alt_loc = node.alternate.loc?.start;
				const con_end = node.consequent.loc?.end;
				if (alt_loc && con_end && con_end.line === alt_loc.line && alt_loc.column >= 4) {
					write_source_keyword(context, con_end.line, con_end.column + 1, 'else');
					context.write(' ');
				} else {
					context.write('else ');
				}

				context.visit(node.alternate);
			}
		},

		ImportDeclaration(node, context) {
			if (node.specifiers.length === 0) {
				write_keyword(context, node, 'import', ' ');
				context.visit(node.source);
				context.write(';');
				return;
			}

			/** @type {TSESTree.ImportNamespaceSpecifier | null} */
			let namespace_specifier = null;

			/** @type {TSESTree.ImportDefaultSpecifier | null} */
			let default_specifier = null;

			/** @type {TSESTree.ImportSpecifier[]} */
			const named_specifiers = [];

			for (const s of node.specifiers) {
				if (s.type === 'ImportNamespaceSpecifier') {
					namespace_specifier = s;
				} else if (s.type === 'ImportDefaultSpecifier') {
					default_specifier = s;
				} else {
					named_specifiers.push(s);
				}
			}

			const kw = create_keyword_write(context, node);

			kw('import ');
			if (node.importKind == 'type') {
				if (single_line_node(node)) kw('type ');
				else context.write('type ');
			}

			if (default_specifier) {
				context.write(default_specifier.local.name, default_specifier);
				if (namespace_specifier || named_specifiers.length > 0) context.write(', ');
			}

			if (namespace_specifier) {
				context.write('* as ' + namespace_specifier.local.name, namespace_specifier);
			}

			if (named_specifiers.length > 0) {
				context.write('{');
				sequence(context, named_specifiers, node.source.loc?.start ?? null, true);
				context.write('}');
			}

			context.write(' from ');
			context.visit(node.source);
			if (node.attributes && node.attributes.length > 0) {
				context.write(' with { ');
				for (let index = 0; index < node.attributes.length; index++) {
					const { key, value } = node.attributes[index];
					context.visit(key);
					context.write(': ');
					context.visit(value);
					if (index + 1 !== node.attributes.length) {
						context.write(', ');
					}
				}
				context.write(' }');
			}
			context.write(';');
		},

		ImportExpression(node, context) {
			write_keyword(context, node, 'import', '(');
			context.visit(node.source);
			//@ts-expect-error for some reason the types haven't been updated
			if (node.arguments) {
				//@ts-expect-error
				for (let index = 0; index < node.arguments.length; index++) {
					context.write(', ');
					//@ts-expect-error
					context.visit(node.arguments[index]);
				}
			}
			if (node.options) {
				context.write(', ');
				context.visit(node.options);
			}
			context.write(')');
		},

		ImportSpecifier(node, context) {
			if (node.importKind == 'type') context.write('type ');

			if (
				node.local.type === 'Identifier' &&
				node.imported.type === 'Identifier' &&
				node.local.name !== node.imported.name
			) {
				context.visit(node.imported);
				context.write(' as ');
			}

			context.visit(node.local);
		},

		LabeledStatement(node, context) {
			context.visit(node.label);
			context.write(': ');
			context.visit(node.body);
		},

		Literal(node, context) {
			// TODO do we need to handle weird unicode characters somehow?
			// str.replace(/\\u(\d{4})/g, (m, n) => String.fromCharCode(+n))

			const value =
				node.raw ||
				(typeof node.value === 'string' ? quote(node.value, quote_char) : String(node.value));

			context.write(value, node);
		},

		LogicalExpression: shared['BinaryExpression|LogicalExpression'],

		MemberExpression(node, context) {
			const wrap =
				node.object.type === 'ChainExpression' ||
				EXPRESSIONS_PRECEDENCE[node.object.type] < EXPRESSIONS_PRECEDENCE.MemberExpression;
			maybe_wrap(context, node.object, wrap);

			if (node.computed) {
				if (node.optional) {
					context.write('?.');
				}
				context.write('[');
				context.visit(node.property);
				context.write(']', token_before(node.loc?.end));
			} else {
				context.write(node.optional ? '?.' : '.');
				context.visit(node.property);
			}
		},

		MetaProperty(node, context) {
			context.visit(node.meta);
			context.write('.');
			context.visit(node.property);
		},

		MethodDefinition: shared['MethodDefinition|TSAbstractMethodDefinition'],

		NewExpression: shared['CallExpression|NewExpression'],

		ObjectExpression(node, context) {
			context.write('{', token_at(node.loc?.start));
			sequence(context, node.properties, node.loc?.end ?? null, true);
			context.write('}', token_before(node.loc?.end));
		},

		ObjectPattern(node, context) {
			context.write('{', token_at(node.loc?.start));
			sequence(context, node.properties, node.loc?.end ?? null, true);
			context.write('}', token_before(node.loc?.end));

			if (node.typeAnnotation) context.visit(node.typeAnnotation);
		},

		// @ts-expect-error this isn't a real node type, but Acorn produces it
		ParenthesizedExpression(node, context) {
			if (node.loc) {
				context.write('(', token_at(node.loc.start));
				context.visit(node.expression);
				context.write(')', token_before(node.loc.end));
			} else {
				maybe_wrap(context, node.expression, true);
			}
		},

		PrivateIdentifier(node, context) {
			context.write('#');
			context.write(node.name, node);
		},

		Program(node, context) {
			body(context, node);
		},

		Property(node, context) {
			const value = node.value.type === 'AssignmentPattern' ? node.value.left : node.value;

			const shorthand =
				!node.computed &&
				node.kind === 'init' &&
				node.key.type === 'Identifier' &&
				value.type === 'Identifier' &&
				node.key.name === value.name;

			if (shorthand) {
				context.visit(node.value);
				return;
			}

			// shorthand methods
			if (node.value.type === 'FunctionExpression') {
				const kw = create_keyword_write(context, node);

				if (node.kind !== 'init') kw(node.kind + ' ');
				if (node.value.async) kw('async ');
				if (node.value.generator) context.write('*');
				if (node.computed) context.write('[', token_before(node.key.loc?.start));
				context.visit(node.key);
				if (node.computed) context.write(']', token_at(node.key.loc?.end));
				context.write('(');
				sequence(
					context,
					node.value.params,
					(node.value.returnType ?? node.value.body).loc?.start ?? null,
					false
				);
				context.write(')');

				if (node.value.returnType) context.visit(node.value.returnType);

				context.write(' ');
				context.visit(node.value.body);
			} else {
				if (node.computed) context.write('[', token_before(node.key.loc?.start));
				if (node.kind === 'get' || node.kind === 'set') {
					write_keyword(context, node, node.kind, ' ');
				}
				context.visit(node.key);
				if (node.computed) {
					context.write(']', token_at(node.key.loc?.end));
					context.write(': ');
				} else {
					context.write(': ');
				}
				context.visit(node.value);
			}
		},

		PropertyDefinition:
			shared[
				'PropertyDefinition|TSAbstractPropertyDefinition|AccessorProperty|TSAbstractAccessorProperty'
			],

		RestElement: shared['RestElement|SpreadElement'],

		ReturnStatement(node, context) {
			if (node.argument) {
				const contains_comment =
					comments[comment_index] &&
					comments[comment_index].loc &&
					node.argument.loc &&
					before(comments[comment_index].loc.start, node.argument.loc.start);

				write_keyword(context, node, 'return', contains_comment ? ' (' : ' ');
				context.visit(node.argument);
				context.write(contains_comment ? ');' : ';');
			} else {
				write_keyword(context, node, 'return', ';');
			}
		},

		SequenceExpression(node, context) {
			context.write('(');
			sequence(context, node.expressions, node.loc?.end ?? null, false);
			context.write(')');
		},

		SpreadElement: shared['RestElement|SpreadElement'],

		StaticBlock(node, context) {
			write_keyword(context, node, 'static', ' {');
			context.indent();
			context.newline();

			body(context, node);

			context.dedent();
			context.newline();
			context.write('}');
		},

		Super(node, context) {
			context.write('super', node);
		},

		SwitchStatement(node, context) {
			write_keyword(context, node, 'switch', ' (');
			context.visit(node.discriminant);
			context.write(') {');
			context.indent();

			let first = true;

			for (const block of node.cases) {
				if (!first) {
					context.margin();
				}

				first = false;

				if (block.test) {
					context.newline();
					write_keyword(context, block, 'case', ' ');
					context.visit(block.test);
					context.write(':');
				} else {
					context.newline();
					write_keyword(context, block, 'default', ':');
				}

				context.indent();

				for (const statement of block.consequent) {
					context.newline();
					context.visit(statement);
				}

				context.dedent();
			}

			context.dedent();
			context.newline();
			context.write('}');
		},

		TaggedTemplateExpression(node, context) {
			// the tag is a LeftHandSideExpression; a lower-precedence tag (logical,
			// conditional, arrow, `as`, unary…) or an optional chain must be wrapped
			const wrap =
				/** @type {string} */ (node.tag.type) === 'ChainExpression' ||
				EXPRESSIONS_PRECEDENCE[node.tag.type] < EXPRESSIONS_PRECEDENCE.CallExpression;
			maybe_wrap(context, node.tag, wrap);
			context.visit(node.quasi);
		},

		TemplateLiteral(node, context) {
			context.write('`');

			const { quasis, expressions } = node;

			for (let i = 0; i < expressions.length; i++) {
				const raw = quasis[i].value.raw;

				context.write(raw + '${');
				context.visit(expressions[i]);
				context.write('}');

				if (/\n/.test(raw)) context.multiline = true;
			}

			const raw = quasis[quasis.length - 1].value.raw;

			context.write(raw + '`');
			if (/\n/.test(raw)) context.multiline = true;
		},

		ThisExpression(node, context) {
			context.write('this', node);
		},

		ThrowStatement(node, context) {
			write_keyword(context, node, 'throw', ' ');
			if (node.argument) context.visit(node.argument);
			context.write(';');
		},

		TryStatement(node, context) {
			write_keyword(context, node, 'try', ' ');
			context.visit(node.block);

			if (node.handler) {
				context.write(' ');

				if (node.handler.param) {
					write_keyword(context, node.handler, 'catch', '(');
					context.visit(node.handler.param);
					context.write(') ');
				} else {
					write_keyword(context, node.handler, 'catch', ' ');
				}

				context.visit(node.handler.body);
			}

			if (node.finalizer) {
				const fin_loc = node.finalizer.loc?.start;
				const prev_end = node.handler ? node.handler.loc?.end : node.block.loc?.end;
				if (fin_loc && prev_end && prev_end.line === fin_loc.line && fin_loc.column >= 7) {
					context.write(' ');
					write_source_keyword(context, prev_end.line, prev_end.column + 1, 'finally');
					context.write(' ');
				} else {
					context.write(' finally ');
				}

				context.visit(node.finalizer);
			}
		},

		UnaryExpression(node, context) {
			context.write(node.operator, token_at(node.loc?.start, node.operator.length));

			if (node.operator.length > 1) {
				context.write(' ');
			} else if (
				(node.operator === '+' || node.operator === '-') &&
				((node.argument.type === 'UnaryExpression' && node.argument.operator === node.operator) ||
					(node.argument.type === 'UpdateExpression' &&
						node.argument.prefix &&
						node.argument.operator[0] === node.operator))
			) {
				context.write(' ');
			}

			const wrap =
				EXPRESSIONS_PRECEDENCE[node.argument.type] < EXPRESSIONS_PRECEDENCE.UnaryExpression;
			maybe_wrap(context, node.argument, wrap);
		},

		UpdateExpression(node, context) {
			if (node.prefix) {
				context.write(node.operator);
				context.visit(node.argument);
			} else {
				context.visit(node.argument);
				context.write(node.operator);
			}
		},

		VariableDeclaration(node, context) {
			handle_var_declaration(node, context);
			context.write(';');
		},

		VariableDeclarator(node, context) {
			context.visit(node.id);

			if (node.init) {
				context.write(' = ');
				context.visit(node.init);
			}
		},

		WhileStatement(node, context) {
			write_keyword(context, node, 'while', ' (');
			context.visit(node.test);
			context.write(') ');
			context.visit(node.body);
		},

		WithStatement(node, context) {
			write_keyword(context, node, 'with', ' (');
			context.visit(node.object);
			context.write(') ');
			context.visit(node.body);
		},

		YieldExpression(node, context) {
			const word = node.delegate ? 'yield*' : 'yield';

			if (node.argument) {
				write_keyword(context, node, word, ' ');
				context.visit(node.argument);
			} else {
				write_keyword(context, node, word, '');
			}
		},

		TSAbstractMethodDefinition: shared['MethodDefinition|TSAbstractMethodDefinition'],

		TSAbstractAccessorProperty:
			shared[
				'PropertyDefinition|TSAbstractPropertyDefinition|AccessorProperty|TSAbstractAccessorProperty'
			],

		TSAbstractPropertyDefinition:
			shared[
				'PropertyDefinition|TSAbstractPropertyDefinition|AccessorProperty|TSAbstractAccessorProperty'
			],

		TSDeclareFunction(node, context) {
			const kw = create_keyword_write(context, node);

			kw('declare ');

			if (node.async) {
				kw('async ');
			}

			kw('function');

			if (node.generator) {
				context.write('*');
			}

			if (node.id) {
				context.write(' ');
				context.visit(node.id);
			}

			if (node.typeParameters) {
				context.visit(node.typeParameters);
			}

			context.write('(');
			sequence(context, node.params, node.returnType?.loc?.start ?? node.loc?.end ?? null, false);
			context.write(')');

			if (node.returnType) {
				context.visit(node.returnType);
			}

			context.write(';');
		},

		TSNumberKeyword(node, context) {
			context.write('number', node);
		},

		TSStringKeyword(node, context) {
			context.write('string', node);
		},

		TSBooleanKeyword(node, context) {
			context.write('boolean', node);
		},

		TSAnyKeyword(node, context) {
			context.write('any', node);
		},

		TSVoidKeyword(node, context) {
			context.write('void', node);
		},

		TSUnknownKeyword(node, context) {
			context.write('unknown', node);
		},

		TSNeverKeyword(node, context) {
			context.write('never', node);
		},

		TSSymbolKeyword(node, context) {
			context.write('symbol', node);
		},

		TSNullKeyword(node, context) {
			context.write('null', node);
		},

		TSUndefinedKeyword(node, context) {
			context.write('undefined', node);
		},

		TSObjectKeyword(node, context) {
			context.write('object', node);
		},

		TSBigIntKeyword(node, context) {
			context.write('bigint', node);
		},

		TSIntrinsicKeyword(node, context) {
			context.write('intrinsic', node);
		},

		TSArrayType(node, context) {
			context.visit(node.elementType);
			context.write('[]');
		},

		TSTypeAnnotation(node, context) {
			context.write(': ');
			context.visit(node.typeAnnotation);
		},

		TSTypeLiteral(node, context) {
			context.write('{ ');
			sequence(context, node.members, node.loc?.end ?? null, false, ';');
			context.write(' }');
		},

		TSPropertySignature(node, context) {
			if (node.computed) context.write('[', token_before(node.key.loc?.start));
			context.visit(node.key);
			if (node.computed) context.write(']', token_at(node.key.loc?.end));
			if (node.optional) context.write('?');
			if (node.typeAnnotation) context.visit(node.typeAnnotation);
		},

		TSTypeReference(node, context) {
			context.visit(node.typeName);

			if (node.typeArguments) {
				context.visit(node.typeArguments);
			}
		},

		TSTypeOperator(node, context) {
			context.write(node.operator + ' ');
			if (node.typeAnnotation) {
				context.visit(node.typeAnnotation);
			}
		},

		TSTemplateLiteralType(node, context) {
			context.write('`');
			const { quasis, types } = node;
			for (let i = 0; i < types.length; i++) {
				const raw = quasis[i].value.raw;

				context.write(raw + '${');
				context.visit(types[i]);
				context.write('}');

				if (/\n/.test(raw)) context.multiline = true;
			}
			context.write('`');
		},

		TSParameterProperty(node, context) {
			if (node.accessibility) {
				context.write(node.accessibility + ' ');
			}

			if (node.readonly) {
				context.write('readonly ');
			}

			context.visit(node.parameter);
		},

		TSExportAssignment(node, context) {
			context.write('export = ');
			context.visit(node.expression);
			context.write(';');
		},

		TSNamespaceExportDeclaration(node, context) {
			context.write('export as namespace ');
			context.visit(node.id);
			context.write(';');
		},

		//@ts-expect-error I don't know why, but this is relied upon in the tests, but doesn't exist in the TSESTree types
		TSExpressionWithTypeArguments(node, context) {
			context.visit(node.expression);
		},

		TSTypeAssertion(node, context) {
			context.write('<');
			context.visit(node.typeAnnotation);
			context.write('>');
			const wrap =
				EXPRESSIONS_PRECEDENCE[node.expression.type] < EXPRESSIONS_PRECEDENCE.TSTypeAssertion;
			maybe_wrap(context, node.expression, wrap);
		},

		TSTypeParameterInstantiation(node, context) {
			context.write('<');
			for (let i = 0; i < node.params.length; i++) {
				context.visit(node.params[i]);
				if (i != node.params.length - 1) context.write(', ');
			}
			context.write('>');
		},

		TSTypeParameterDeclaration(node, context) {
			context.write('<');
			for (let i = 0; i < node.params.length; i++) {
				context.visit(node.params[i]);
				if (i != node.params.length - 1) context.write(', ');
			}
			context.write('>');
		},

		TSTypeParameter(node, context) {
			// modifiers: `const T`, `in T` / `out T` (variance)
			if (node.const) context.write('const ');
			if (node.in) context.write('in ');
			if (node.out) context.write('out ');

			if (node.name && node.name.type) context.visit(node.name);
			// @ts-expect-error type mismatch TSESTree and acorn-typescript?
			else context.write(node.name, node);

			if (node.constraint) {
				context.write(' extends ');
				context.visit(node.constraint);
			}

			if (node.default) {
				context.write(' = ');
				context.visit(node.default);
			}
		},

		TSTypePredicate(node, context) {
			// `asserts` precedes the parameter name; `is <type>` follows it. Forms:
			// `x is T`, `asserts x is T`, `asserts x`
			if (node.asserts) context.write('asserts ');

			if (node.parameterName) context.visit(node.parameterName);

			if (node.typeAnnotation) {
				context.write(' is ');
				context.visit(node.typeAnnotation.typeAnnotation);
			}
		},

		TSTypeQuery(node, context) {
			context.write('typeof ');
			context.visit(node.exprName);
		},

		TSClassImplements(node, context) {
			if (node.expression) {
				context.visit(node.expression);
			}
		},

		TSEnumMember(node, context) {
			context.visit(node.id);
			if (node.initializer) {
				context.write(' = ');
				context.visit(node.initializer);
			}
		},

		TSFunctionType: shared['TSFunctionType|TSConstructorType'],

		TSIndexSignature(node, context) {
			context.write('[');

			// @ts-expect-error `acorn-typescript` and `@typescript-eslint/types` have slightly different type definitions
			sequence(context, node.parameters, node.typeAnnotation?.loc?.start ?? null, false);
			context.write(']');

			// @ts-expect-error `acorn-typescript` and `@typescript-eslint/types` have slightly different type definitions
			context.visit(node.typeAnnotation);
		},

		TSMappedType(node, context) {
			context.write('{');

			// `readonly` / `+readonly` / `-readonly` modifier
			if (node.readonly) {
				context.write(
					node.readonly === '-' ? '-readonly ' : node.readonly === '+' ? '+readonly ' : 'readonly '
				);
			}

			context.write('[');

			const legacy_type_parameter = node.typeParameter;
			const key = node.key ?? legacy_type_parameter?.name;
			const constraint = node.constraint ?? legacy_type_parameter?.constraint;

			if (key && typeof key === 'object') {
				context.visit(key);
			} else {
				context.write(key, node);
			}

			if (constraint) {
				context.write(' in ');
				context.visit(constraint);
			}

			// `as` key remapping
			if (node.nameType) {
				context.write(' as ');
				context.visit(node.nameType);
			}

			context.write(']');

			// `?` / `+?` / `-?` optionality modifier
			if (node.optional) {
				context.write(node.optional === '-' ? '-?' : node.optional === '+' ? '+?' : '?');
			}

			if (node.typeAnnotation) {
				context.write(': ');
				context.visit(node.typeAnnotation);
			}

			context.write('}');
		},

		TSMethodSignature(node, context) {
			if (node.computed) context.write('[', token_before(node.key.loc?.start));
			context.visit(node.key);
			if (node.computed) context.write(']', token_at(node.key.loc?.end));

			context.write('(');
			sequence(
				context,
				// @ts-expect-error `acorn-typescript` and `@typescript-eslint/types` have slightly different type definitions
				node.parameters ?? node.params,
				// @ts-expect-error `acorn-typescript` and `@typescript-eslint/types` have slightly different type definitions
				(node.typeAnnotation ?? node.returnType)?.loc?.start ?? null,
				false
			);
			context.write(')');

			// @ts-expect-error `acorn-typescript` and `@typescript-eslint/types` have slightly different type definitions
			if (node.typeAnnotation || node.returnType) {
				// @ts-expect-error `acorn-typescript` and `@typescript-eslint/types` have slightly different type definitions
				context.visit(node.typeAnnotation ?? node.returnType);
			}
		},

		TSTupleType(node, context) {
			context.write('[');
			sequence(context, node.elementTypes, node.loc?.end ?? null, false);
			context.write(']');
		},

		TSNamedTupleMember(node, context) {
			context.visit(node.label);
			context.write(': ');
			context.visit(node.elementType);
		},

		TSUnionType(node, context) {
			// no trailing newline, so a following `=>` stays on the same line
			sequence(context, node.types, node.loc?.end ?? null, false, ' |', false);
		},

		TSIntersectionType(node, context) {
			// no trailing newline, so a following `=>` stays on the same line
			sequence(context, node.types, node.loc?.end ?? null, false, ' &', false);
		},

		TSInferType(node, context) {
			context.write('infer ');
			context.visit(node.typeParameter);
		},

		TSLiteralType(node, context) {
			context.visit(node.literal);
		},

		TSCallSignatureDeclaration:
			shared['TSConstructSignatureDeclaration|TSCallSignatureDeclaration'],

		TSConditionalType(node, context) {
			context.visit(node.checkType);
			context.write(' extends ');
			context.visit(node.extendsType);
			context.write(' ? ');
			context.visit(node.trueType);
			context.write(' : ');
			context.visit(node.falseType);
		},

		TSConstructSignatureDeclaration:
			shared['TSConstructSignatureDeclaration|TSCallSignatureDeclaration'],

		TSConstructorType: shared['TSFunctionType|TSConstructorType'],

		TSExternalModuleReference(node, context) {
			context.write('require(');
			context.visit(node.expression);
			context.write(');');
		},

		TSIndexedAccessType(node, context) {
			context.visit(node.objectType);
			context.write('[');
			context.visit(node.indexType);
			context.write(']');
		},

		TSImportEqualsDeclaration(node, context) {
			context.write('import ');
			context.visit(node.id);
			context.write(' = ');
			context.visit(node.moduleReference);
		},

		TSImportType(node, context) {
			context.write('import(');
			context.visit(node.argument);
			context.write(')');

			if (node.qualifier) {
				context.write('.');
				context.visit(node.qualifier);
			}
		},

		TSOptionalType(node, context) {
			context.visit(node.typeAnnotation);
			context.write('?');
		},

		TSRestType(node, context) {
			context.write('...');
			context.visit(node.typeAnnotation);
		},

		TSThisType(node, context) {
			context.write('this', node);
		},

		TSAsExpression(node, context) {
			if (node.expression) {
				const wrap =
					EXPRESSIONS_PRECEDENCE[node.expression.type] < EXPRESSIONS_PRECEDENCE.TSAsExpression;
				maybe_wrap(context, node.expression, wrap);
			}
			context.write(' as ');
			context.visit(node.typeAnnotation);
		},

		TSEnumDeclaration(node, context) {
			context.write('enum ');
			context.visit(node.id);
			context.write(' {');
			context.indent();
			context.newline();
			sequence(context, node.members ?? node.body.members, node.loc?.end ?? null, false);
			context.dedent();
			context.newline();
			context.write('}');
		},

		TSModuleBlock(node, context) {
			context.write(' {');
			context.indent();
			context.newline();
			body(context, node);
			context.dedent();
			context.newline();
			context.write('}');
		},

		TSModuleDeclaration(node, context) {
			if (node.declare) context.write('declare ');

			if (node.global) {
				context.write('global', node.id);
			} else {
				// @ts-expect-error `acorn-typescript` and `@typescript-eslint/types` have slightly different type definitions
				const kind = node.kind ?? (node.id.type === 'Literal' ? 'module' : 'namespace');
				context.write(kind + ' ');
				context.visit(node.id);
			}

			// a qualified name (`namespace A.B.C`) is represented as nested
			// `TSModuleDeclaration`s whose body is the next name part, not a block
			let body = /** @type {any} */ (node.body);
			while (body && body.type === 'TSModuleDeclaration') {
				context.write('.');
				context.visit(body.id);
				body = body.body;
			}

			if (!body) return;
			context.visit(body);
		},

		TSNonNullExpression(node, context) {
			// operator expressions can't take a postfix `!` directly: `(0 as number)!`, `(await x)!`
			const wrap =
				EXPRESSIONS_PRECEDENCE[node.expression.type] < EXPRESSIONS_PRECEDENCE.TSNonNullExpression;
			maybe_wrap(context, node.expression, wrap);
			context.write('!');
		},

		TSInterfaceBody(node, context) {
			sequence(context, node.body, node.loc?.end ?? null, true, ';');
		},

		TSInterfaceDeclaration(node, context) {
			context.write('interface ');
			context.visit(node.id);
			if (node.typeParameters) context.visit(node.typeParameters);
			if (node.extends && node.extends.length > 0) {
				context.write(' extends ');
				sequence(context, node.extends, node.body.loc?.start ?? null, false);
			}
			context.write(' {');
			context.visit(node.body);
			context.write('}');
		},

		TSInstantiationExpression(node, context) {
			context.visit(node.expression);
			context.visit(node.typeArguments);
		},

		TSInterfaceHeritage(node, context) {
			if (node.expression) {
				context.visit(node.expression);
			}
		},

		//@ts-expect-error I don't know why, but this is relied upon in the tests, but doesn't exist in the TSESTree types
		TSParenthesizedType(node, context) {
			maybe_wrap(context, node.typeAnnotation, true);
		},

		TSSatisfiesExpression(node, context) {
			if (node.expression) {
				const wrap =
					EXPRESSIONS_PRECEDENCE[node.expression.type] <
					EXPRESSIONS_PRECEDENCE.TSSatisfiesExpression;
				maybe_wrap(context, node.expression, wrap);
			}
			context.write(' satisfies ');
			context.visit(node.typeAnnotation);
		},

		TSTypeAliasDeclaration(node, context) {
			context.write('type ');
			context.visit(node.id);
			if (node.typeParameters) context.visit(node.typeParameters);
			context.write(' = ');
			context.visit(node.typeAnnotation);
			context.write(';');
		},

		TSQualifiedName(node, context) {
			context.visit(node.left);
			context.write('.');
			context.visit(node.right);
		}
	};
};

/** @satisfies {Visitors} */

/**
 * Arrow functions with a concise body must wrap certain expressions in parentheses,
 * otherwise `{` can start a block statement instead of an object literal (`as` /
 * `satisfies` / `!` do not change that (e.g. `() => { x } as const`).
 * @param {TSESTree.BlockStatement | TSESTree.Expression} body
 * @returns {boolean}
 */
function arrow_concise_body_needs_wrap(body) {
	if (body.type === 'BlockStatement') return false;

	switch (body.type) {
		case 'ObjectExpression':
			return true;
		case 'AssignmentExpression':
			return body.left.type === 'ObjectPattern';
		case 'LogicalExpression':
			return body.left.type === 'ObjectExpression';
		case 'ConditionalExpression':
			return body.test.type === 'ObjectExpression';
		case 'TSAsExpression':
		case 'TSSatisfiesExpression':
		case 'TSNonNullExpression':
			return body.expression ? arrow_concise_body_needs_wrap(body.expression) : false;
		default:
			return false;
	}
}

/**
 *
 * @param {TSESTree.Expression | TSESTree.PrivateIdentifier} node
 * @param {TSESTree.BinaryExpression | TSESTree.LogicalExpression} parent
 * @param {boolean} is_right
 * @returns
 */
function operand_needs_wrap(node, parent, is_right) {
	if (node.type === 'PrivateIdentifier') return false;

	if (!is_right && (node.type === 'TSAsExpression' || node.type === 'TSSatisfiesExpression')) {
		// `**` would be invalid, `&`/`|` would be swallowed into the trailing type
		return parent.operator === '**' || parent.operator === '&' || parent.operator === '|';
	}

	// special case where logical expressions and coalesce expressions cannot be mixed,
	// either of them need to be wrapped with parentheses
	if (
		node.type === 'LogicalExpression' &&
		parent.type === 'LogicalExpression' &&
		((parent.operator === '??' && node.operator !== '??') ||
			(parent.operator !== '??' && node.operator === '??'))
	) {
		return true;
	}

	// `**` can't take a unary/await/assertion left operand: `-2 ** 2`, `await x ** 2`,
	// `<T>x ** 2` are syntax errors
	const unary_base_of_pow =
		!is_right &&
		parent.operator === '**' &&
		(node.type === 'UnaryExpression' ||
			node.type === 'AwaitExpression' ||
			node.type === 'TSTypeAssertion');
	if (unary_base_of_pow) return true;

	const precedence = EXPRESSIONS_PRECEDENCE[node.type];
	const parent_precedence = EXPRESSIONS_PRECEDENCE[parent.type];
	if (precedence !== parent_precedence) return precedence < parent_precedence;
	const operator = /** @type {TSESTree.BinaryExpression} */ (node).operator;
	if (operator === '**' && parent.operator === '**') {
		// exponentiation is right-associative
		return !is_right;
	}

	if (is_right) {
		// parentheses are needed when both operators have the same precedence
		return OPERATOR_PRECEDENCE[operator] <= OPERATOR_PRECEDENCE[parent.operator];
	}

	return OPERATOR_PRECEDENCE[operator] < OPERATOR_PRECEDENCE[parent.operator];
}

/**
 * @param {Context} context
 * @param {TSESTree.Node} node
 * @param {boolean} wrap
 */
function maybe_wrap(context, node, wrap) {
	if (wrap) {
		context.write('(');
		context.visit(node);
		context.write(')');
	} else {
		context.visit(node);
	}
}

/** @param {TSESTree.Node} node */
function has_call_expression(node) {
	while (node) {
		if (node.type === 'CallExpression') {
			return true;
		} else if (node.type === 'MemberExpression') {
			node = node.object;
		} else {
			return false;
		}
	}
	return false;
}

/**
 * True when printing `node` as an expression statement would begin with `{`,
 * `function`, or `class` — which the parser would misread as a block, function
 * declaration, or class declaration. Walks the left spine following the same
 * parenthesization the visitors apply, so it stops as soon as a child position
 * would already be wrapped.
 * @param {TSESTree.Node} node
 * @returns {boolean}
 */
function leads_with_curly_or_keyword(node) {
	while (node) {
		switch (node.type) {
			case 'ObjectExpression':
			case 'ObjectPattern':
			case 'FunctionExpression':
			case 'ClassExpression':
				return true;

			case 'BinaryExpression':
			case 'LogicalExpression':
				if (operand_needs_wrap(node.left, node, false)) return false;
				node = node.left;
				continue;

			case 'AssignmentExpression':
				node = node.left;
				continue;

			case 'ConditionalExpression':
				if (
					EXPRESSIONS_PRECEDENCE[node.test.type] <= EXPRESSIONS_PRECEDENCE.ConditionalExpression
				) {
					return false;
				}
				node = node.test;
				continue;

			case 'MemberExpression':
				if (
					/** @type {string} */ (node.object.type) === 'ChainExpression' ||
					EXPRESSIONS_PRECEDENCE[node.object.type] < EXPRESSIONS_PRECEDENCE.MemberExpression
				) {
					return false;
				}
				node = node.object;
				continue;

			case 'CallExpression':
				if (
					/** @type {string} */ (node.callee.type) === 'ChainExpression' ||
					EXPRESSIONS_PRECEDENCE[node.callee.type] < EXPRESSIONS_PRECEDENCE.CallExpression
				) {
					return false;
				}
				node = node.callee;
				continue;

			case 'TaggedTemplateExpression':
				if (
					/** @type {string} */ (node.tag.type) === 'ChainExpression' ||
					EXPRESSIONS_PRECEDENCE[node.tag.type] < EXPRESSIONS_PRECEDENCE.CallExpression
				) {
					return false;
				}
				node = node.tag;
				continue;

			case 'UpdateExpression':
				if (node.prefix) return false;
				node = node.argument;
				continue;

			case 'TSAsExpression':
			case 'TSSatisfiesExpression':
			case 'TSNonNullExpression':
				node = node.expression;
				continue;

			// a sequence expression always prints its own wrapping parens
			case 'SequenceExpression':
				return false;

			default:
				return false;
		}
	}
	return false;
}

/**
 * @param {TSESTree.VariableDeclaration} node
 * @param {Context} context
 */
function handle_var_declaration(node, context) {
	const open = context.new();
	const join = context.new();
	const child_context = context.new();

	context.append(child_context);

	const kw = create_keyword_write(child_context, node);

	if (node.declare) kw('declare ');
	kw(node.kind + ' ');

	child_context.append(open);

	let first = true;

	for (const d of node.declarations) {
		if (!first) child_context.append(join);
		first = false;

		child_context.visit(d);
	}

	const length = child_context.measure() + 2 * (node.declarations.length - 1);

	const multiline = child_context.multiline || (node.declarations.length > 1 && length > 50);

	if (multiline) {
		context.multiline = true;

		if (node.declarations.length > 1) open.indent();
		join.write(',');
		join.newline();
		if (node.declarations.length > 1) context.dedent();
	} else {
		join.write(', ');
	}
}

/**
 * @param {string} string
 * @param {string} char
 */
function quote(string, char) {
	let out = char;

	for (const c of string) {
		if (c === '\\') {
			out += '\\\\';
		} else if (c === char) {
			out += '\\' + c;
		} else if (c === '\n') {
			out += '\\n';
		} else if (c === '\r') {
			out += '\\r';
		} else {
			out += c;
		}
	}

	return out + char;
}

/**
 *
 * @param {{ line: number, column: number }} a
 * @param {{ line: number, column: number }} b
 */
function before(a, b) {
	if (a.line < b.line) return true;
	if (a.line > b.line) return false;
	return a.column < b.column;
}
