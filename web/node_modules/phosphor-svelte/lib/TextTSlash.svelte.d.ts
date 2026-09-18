import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `TextTSlashIcon` instead.
 *
 * @example
 * ```svelte
 * <TextTSlash color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const TextTSlash: Component<IconComponentProps, {}, "">;
type TextTSlash = ReturnType<typeof TextTSlash>;
export default TextTSlash;
