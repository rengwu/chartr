import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `QuestionIcon` instead.
 *
 * @example
 * ```svelte
 * <Question color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Question: Component<IconComponentProps, {}, "">;
type Question = ReturnType<typeof Question>;
export default Question;
