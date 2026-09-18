import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `FlyingSaucerIcon` instead.
 *
 * @example
 * ```svelte
 * <FlyingSaucer color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const FlyingSaucer: Component<IconComponentProps, {}, "">;
type FlyingSaucer = ReturnType<typeof FlyingSaucer>;
export default FlyingSaucer;
