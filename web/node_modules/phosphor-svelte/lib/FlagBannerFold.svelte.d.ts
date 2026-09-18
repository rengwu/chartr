import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `FlagBannerFoldIcon` instead.
 *
 * @example
 * ```svelte
 * <FlagBannerFold color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const FlagBannerFold: Component<IconComponentProps, {}, "">;
type FlagBannerFold = ReturnType<typeof FlagBannerFold>;
export default FlagBannerFold;
