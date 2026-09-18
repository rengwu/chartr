import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `SignOutIcon` instead.
 *
 * @example
 * ```svelte
 * <SignOut color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const SignOut: Component<IconComponentProps, {}, "">;
type SignOut = ReturnType<typeof SignOut>;
export default SignOut;
