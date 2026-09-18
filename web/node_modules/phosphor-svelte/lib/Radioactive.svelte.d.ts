import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `RadioactiveIcon` instead.
 *
 * @example
 * ```svelte
 * <Radioactive color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Radioactive: Component<IconComponentProps, {}, "">;
type Radioactive = ReturnType<typeof Radioactive>;
export default Radioactive;
