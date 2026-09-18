import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `EyeIcon` instead.
 *
 * @example
 * ```svelte
 * <Eye color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Eye: Component<IconComponentProps, {}, "">;
type Eye = ReturnType<typeof Eye>;
export default Eye;
