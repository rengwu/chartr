import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `SealQuestionIcon` instead.
 *
 * @example
 * ```svelte
 * <SealQuestion color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const SealQuestion: Component<IconComponentProps, {}, "">;
type SealQuestion = ReturnType<typeof SealQuestion>;
export default SealQuestion;
