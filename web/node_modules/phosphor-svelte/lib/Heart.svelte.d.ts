import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `HeartIcon` instead.
 *
 * @example
 * ```svelte
 * <Heart color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Heart: Component<IconComponentProps, {}, "">;
type Heart = ReturnType<typeof Heart>;
export default Heart;
