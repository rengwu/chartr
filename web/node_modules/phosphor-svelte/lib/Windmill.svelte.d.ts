import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `WindmillIcon` instead.
 *
 * @example
 * ```svelte
 * <Windmill color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Windmill: Component<IconComponentProps, {}, "">;
type Windmill = ReturnType<typeof Windmill>;
export default Windmill;
