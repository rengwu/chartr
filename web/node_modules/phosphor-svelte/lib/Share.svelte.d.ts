import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ShareIcon` instead.
 *
 * @example
 * ```svelte
 * <Share color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Share: Component<IconComponentProps, {}, "">;
type Share = ReturnType<typeof Share>;
export default Share;
