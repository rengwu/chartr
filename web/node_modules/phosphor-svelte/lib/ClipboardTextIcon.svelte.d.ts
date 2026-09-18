import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @example
 * ```svelte
 * <ClipboardTextIcon color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ClipboardTextIcon: Component<IconComponentProps, {}, "">;
type ClipboardTextIcon = ReturnType<typeof ClipboardTextIcon>;
export default ClipboardTextIcon;
