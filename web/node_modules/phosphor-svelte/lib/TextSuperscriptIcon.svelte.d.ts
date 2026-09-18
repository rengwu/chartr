import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @example
 * ```svelte
 * <TextSuperscriptIcon color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const TextSuperscriptIcon: Component<IconComponentProps, {}, "">;
type TextSuperscriptIcon = ReturnType<typeof TextSuperscriptIcon>;
export default TextSuperscriptIcon;
