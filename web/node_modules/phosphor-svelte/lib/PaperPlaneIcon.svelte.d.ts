import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @example
 * ```svelte
 * <PaperPlaneIcon color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const PaperPlaneIcon: Component<IconComponentProps, {}, "">;
type PaperPlaneIcon = ReturnType<typeof PaperPlaneIcon>;
export default PaperPlaneIcon;
