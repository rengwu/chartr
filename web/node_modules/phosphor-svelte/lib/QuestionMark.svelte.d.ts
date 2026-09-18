import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `QuestionMarkIcon` instead.
 *
 * @example
 * ```svelte
 * <QuestionMark color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const QuestionMark: Component<IconComponentProps, {}, "">;
type QuestionMark = ReturnType<typeof QuestionMark>;
export default QuestionMark;
