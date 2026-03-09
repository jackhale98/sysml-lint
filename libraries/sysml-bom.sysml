package SysMLBOM {
    doc /*
     * Domain library for bill-of-materials management.
     * Provides reusable definitions for part identification,
     * cost and mass tracking, and supplier source management.
     */

    private import ScalarValues::*;

    enum def PartCategory {
        doc /* Classifies a part by its role in the product structure. */
        enum assembly;
        enum subassembly;
        enum component;
        enum rawMaterial;
        enum fastener;
        enum consumable;
        enum software;
        enum document;
    }

    enum def LifecycleState {
        doc /* Tracks the maturity stage of a part through its lifecycle. */
        enum concept;
        enum development;
        enum prototype;
        enum production;
        enum obsolete;
    }

    enum def MakeOrBuy {
        doc /* Indicates whether a part is manufactured internally or procured. */
        enum make;
        enum buy;
        enum makeAndBuy;
        enum tbd;
    }

    attribute def PartIdentity {
        doc /*
         * Core identification attributes for a part in the BOM.
         * Every part carries a unique part number, revision,
         * and classification metadata for configuration management.
         */
        attribute partNumber : String;
        attribute revision : String;
        attribute description : String;
        attribute category : PartCategory;
        attribute lifecycleState : LifecycleState;
        attribute makeOrBuy : MakeOrBuy;
    }

    attribute def MassProperty {
        doc /*
         * Mass-related properties for a part or assembly.
         * Captures the measured or estimated mass, the basis
         * for the value, and any margin applied for uncertainty.
         */
        attribute mass : Real;
        attribute massType : String;
        attribute massMargin : Real;
    }

    attribute def CostProperty {
        doc /*
         * Cost attributes for a part or assembly.
         * Captures recurring unit cost, non-recurring tooling
         * cost, and the basis or source of the estimate.
         */
        attribute unitCost : Real;
        attribute toolingCost : Real;
        attribute costBasis : String;
    }

    part def SupplierDef {
        doc /*
         * Represents a supplier organization in the supply chain.
         * Tracks company identity and qualification status for
         * approved vendor list management.
         */
        attribute companyName : String;
        attribute supplierCode : String;
        attribute qualificationStatus : String;
    }

    connection def SourceDef {
        doc /*
         * Links a supplier to a part, capturing procurement
         * details such as lead time, pricing, and whether
         * this is the preferred source for the item.
         */
        attribute supplierPartNumber : String;
        attribute leadTimeDays : Integer;
        attribute unitPrice : Real;
        attribute sourceType : String;
        attribute isPreferred : Boolean;
    }
}
