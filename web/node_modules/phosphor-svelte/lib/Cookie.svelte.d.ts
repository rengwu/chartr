import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `CookieIcon` instead.
 *
 * @example
 * ```svelte
 * <Cookie color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Cookie: Component<IconComponentProps, {}, "">;
type Cookie = ReturnType<typeof Cookie>;
export default Cookie;
