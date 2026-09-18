import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `BeanieIcon` instead.
 *
 * @example
 * ```svelte
 * <Beanie color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Beanie: Component<IconComponentProps, {}, "">;
type Beanie = ReturnType<typeof Beanie>;
export default Beanie;
