import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `FlagBannerIcon` instead.
 *
 * @example
 * ```svelte
 * <FlagBanner color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const FlagBanner: Component<IconComponentProps, {}, "">;
type FlagBanner = ReturnType<typeof FlagBanner>;
export default FlagBanner;
