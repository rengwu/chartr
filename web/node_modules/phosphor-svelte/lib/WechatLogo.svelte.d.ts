import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `WechatLogoIcon` instead.
 *
 * @example
 * ```svelte
 * <WechatLogo color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const WechatLogo: Component<IconComponentProps, {}, "">;
type WechatLogo = ReturnType<typeof WechatLogo>;
export default WechatLogo;
