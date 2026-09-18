import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `BirdIcon` instead.
 *
 * @example
 * ```svelte
 * <Bird color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Bird: Component<IconComponentProps, {}, "">;
type Bird = ReturnType<typeof Bird>;
export default Bird;
