import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `HeadphonesIcon` instead.
 *
 * @example
 * ```svelte
 * <Headphones color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Headphones: Component<IconComponentProps, {}, "">;
type Headphones = ReturnType<typeof Headphones>;
export default Headphones;
