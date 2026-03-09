package SysMLTolerance {
    doc /*
     * Domain library for dimensional tolerance analysis.
     * Provides definitions for specifying tolerances, performing
     * tolerance stack-up analysis, and capturing GD&T per ASME Y14.5.
     */

    private import ScalarValues::*;

    enum def DistributionType {
        doc /* Statistical distribution assumed for a tolerance dimension. */
        enum normal;
        enum uniform;
        enum triangular;
        enum skewedLeft;
        enum skewedRight;
    }

    attribute def ToleranceDef {
        doc /*
         * Base definition for a dimensional tolerance.
         * Specifies the nominal value with upper and lower limits,
         * the assumed statistical distribution, and whether the
         * dimension is on the critical-to-quality list.
         */
        attribute nominal : Real;
        attribute upperLimit : Real;
        attribute lowerLimit : Real;
        attribute distributionType : DistributionType;
        attribute isCritical : Boolean;
    }

    attribute def BilateralToleranceDef :> ToleranceDef {
        doc /*
         * A symmetric tolerance expressed as nominal +/- plusMinus.
         * The upperLimit and lowerLimit inherited from ToleranceDef
         * should equal nominal + plusMinus and nominal - plusMinus
         * respectively.
         */
        attribute plusMinus : Real;
    }

    attribute def UnilateralToleranceDef :> ToleranceDef {
        doc /*
         * A tolerance where deviation is permitted in only one
         * direction from nominal. Either upperLimit equals nominal
         * (negative unilateral) or lowerLimit equals nominal
         * (positive unilateral).
         */
    }

    part def DimensionChainDef {
        doc /*
         * A chain of dimensions for tolerance stack-up analysis.
         * Identifies the closing dimension, the analysis method
         * (e.g., worst-case, RSS, Monte Carlo), and the target
         * process capability index.
         */
        attribute closingDimension : String;
        attribute analysisMethod : String;
        attribute targetCpk : Real;
    }

    enum def GeometricCharacteristic {
        doc /*
         * The 14 geometric characteristic symbols defined by
         * ASME Y14.5 for geometric dimensioning and tolerancing.
         */
        enum straightness;
        enum flatness;
        enum circularity;
        enum cylindricity;
        enum lineProfile;
        enum surfaceProfile;
        enum perpendicularity;
        enum angularity;
        enum parallelism;
        enum position;
        enum concentricity;
        enum symmetry;
        enum circularRunout;
        enum totalRunout;
    }

    enum def MaterialCondition {
        doc /*
         * Material condition modifiers applied to feature
         * tolerances per ASME Y14.5.
         */
        enum RFS;
        enum MMC;
        enum LMC;
    }

    part def FeatureControlFrameDef {
        doc /*
         * Represents a GD&T feature control frame specifying
         * a geometric characteristic, its tolerance zone width,
         * and the applicable material condition modifier.
         */
        attribute characteristic : GeometricCharacteristic;
        attribute toleranceZone : Real;
        attribute materialCondition : MaterialCondition;
    }
}
