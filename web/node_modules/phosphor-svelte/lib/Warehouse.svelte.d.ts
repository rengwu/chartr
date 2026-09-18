import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `WarehouseIcon` instead.
 *
 * @example
 * ```svelte
 * <Warehouse color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Warehouse: Component<IconComponentProps, {}, "">;
type Warehouse = ReturnType<typeof Warehouse>;
export default Warehouse;
