import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @example
 * ```svelte
 * <SealQuestionIcon color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const SealQuestionIcon: Component<IconComponentProps, {}, "">;
type SealQuestionIcon = ReturnType<typeof SealQuestionIcon>;
export default SealQuestionIcon;
