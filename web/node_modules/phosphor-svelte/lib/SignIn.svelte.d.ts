import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `SignInIcon` instead.
 *
 * @example
 * ```svelte
 * <SignIn color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const SignIn: Component<IconComponentProps, {}, "">;
type SignIn = ReturnType<typeof SignIn>;
export default SignIn;
