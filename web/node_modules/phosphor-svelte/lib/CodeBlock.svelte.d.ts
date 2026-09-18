import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `CodeBlockIcon` instead.
 *
 * @example
 * ```svelte
 * <CodeBlock color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const CodeBlock: Component<IconComponentProps, {}, "">;
type CodeBlock = ReturnType<typeof CodeBlock>;
export default CodeBlock;
