import java.io.OutputStream;
import java.io.PrintStream;
import java.util.ArrayList;
import java.util.List;

import tla2sany.drivers.FrontEndException;
import tla2sany.drivers.SANY;
import tla2sany.modanalyzer.SpecObj;
import tla2sany.semantic.ASTConstants;
import tla2sany.semantic.ExprNode;
import tla2sany.semantic.ExprOrOpArgNode;
import tla2sany.semantic.FormalParamNode;
import tla2sany.semantic.LetInNode;
import tla2sany.semantic.ModuleNode;
import tla2sany.semantic.NumeralNode;
import tla2sany.semantic.OpApplNode;
import tla2sany.semantic.OpDeclNode;
import tla2sany.semantic.OpDefNode;
import tla2sany.semantic.SemanticNode;
import tla2sany.semantic.StringNode;
import util.FilenameToStream;
import util.SimpleFilenameToStream;
import util.ToolIO;

/**
 * sany-json: the bridge between tlarc (Rust) and SANY (Java).
 *
 * Runs SANY's full front-end (parse, semantic analysis, level checking,
 * module resolution) on a TLA+ spec and emits the resolved semantic tree as
 * JSON on stdout (schema "tla-ast/v1", see docs/ast-schema.md in the tlarc
 * repo). tlarc deserializes this JSON into its Rust AST types.
 *
 * This shim deliberately depends only on tla2tools.jar — the JSON writer is
 * hand-rolled (see docs/adr/0002). It uses the SANY API shipped in the
 * released jar (SANY.frontEndMain); the newer SanySettings/SanyOutput entry
 * points only exist on the master branch of tlaplus.
 *
 * Usage: java -cp tla2tools.jar:classes SanyJson <spec.tla> [includeDir...]
 * Exit codes: 0 = ok, 1 = SANY parse/analysis failure, 2 = usage, 3 = internal.
 */
public final class SanyJson {

    static final int EXIT_OK = 0;
    static final int EXIT_PARSE = 1;
    static final int EXIT_USAGE = 2;
    static final int EXIT_INTERNAL = 3;

    /** Bump together with the Rust side; see docs/ast-schema.md. */
    static final String SCHEMA = "tla-ast/v1";

    public static void main(final String[] args) {
        final int code = run(args);
        System.out.flush();
        System.exit(code);
    }

    static int run(final String[] args) {
        if (args.length < 1 || "-help".equals(args[0]) || "--help".equals(args[0])) {
            printUsage(System.err);
            return EXIT_USAGE;
        }

        final String specPath = args[0];
        final String[] includeDirs = new String[args.length - 1];
        System.arraycopy(args, 1, includeDirs, 0, includeDirs.length);

        try {
            final ModuleNode root = parseSpec(specPath, includeDirs);
            final JsonWriter w = new JsonWriter();
            writeDocument(w, root);
            System.out.print(w.toString());
            System.out.flush();
            return EXIT_OK;
        } catch (final FrontEndException e) {
            System.err.println("SANY parse/analysis failure: " + e.getMessage());
            return EXIT_PARSE;
        } catch (final RuntimeException e) {
            System.err.println("sany-json internal error: " + e);
            return EXIT_INTERNAL;
        }
    }

    /**
     * Parse and analyze the spec with the API shipped in tla2tools.jar
     * v1.7.4: SANY.frontEndMain performs parsing, semantic analysis, and
     * level checking, returning an error level (0 = success).
     */
    static ModuleNode parseSpec(final String specPath, final String[] includeDirs)
            throws FrontEndException {
        final FilenameToStream fts = new SimpleFilenameToStream(includeDirs);
        final SpecObj spec = new SpecObj(specPath, fts);

        // SANY prints progress chatter ("Parsing file ...") via ToolIO.out /
        // System.out. The bridge contract is "stdout carries only JSON", so
        // both are silenced for the duration of the parse. Semantic errors
        // still reach System.err through the PrintStream we pass below.
        final PrintStream nullOut = new PrintStream(OutputStream.nullOutputStream());
        final PrintStream realSysOut = System.out;
        final PrintStream realToolOut = ToolIO.out;
        System.setOut(nullOut);
        ToolIO.out = nullOut;
        try {
            final int errorLevel = SANY.frontEndMain(spec, specPath, System.err);
            if (errorLevel != 0 || spec.getErrorLevel() != 0) {
                throw new FrontEndException("SANY reported errors for " + specPath);
            }
        } finally {
            System.setOut(realSysOut);
            ToolIO.out = realToolOut;
        }
        return spec.getRootModule();
    }

    // ------------------------------------------------------------------
    // Document: {"schema": "tla-ast/v1", "module": {...}}
    // ------------------------------------------------------------------

    static void writeDocument(final JsonWriter w, final ModuleNode root) {
        w.object();
        w.key("schema").string(SCHEMA);
        w.key("module");
        writeModule(w, root);
        w.endObject();
    }

    static void writeModule(final JsonWriter w, final ModuleNode mod) {
        w.object();
        w.key("name").string(mod.getName().toString());

        w.key("constants");
        w.array();
        for (final OpDeclNode decl : mod.getConstantDecls()) {
            writeOpDecl(w, decl);
        }
        w.endArray();

        w.key("variables");
        w.array();
        for (final OpDeclNode decl : mod.getVariableDecls()) {
            writeOpDecl(w, decl);
        }
        w.endArray();

        w.key("operators");
        w.array();
        for (final OpDefNode def : mod.getOpDefs()) {
            writeOpDef(w, def);
        }
        w.endArray();

        w.endObject();
    }

    static void writeOpDecl(final JsonWriter w, final OpDeclNode decl) {
        w.object();
        w.key("name").string(decl.getName().toString());
        w.key("kind").string(kindName(decl.getKind()));
        w.endObject();
    }

    static void writeOpDef(final JsonWriter w, final OpDefNode def) {
        w.object();
        w.key("name").string(def.getName().toString());
        w.key("arity").number(def.getArity());
        w.key("params");
        w.array();
        final FormalParamNode[] params = def.getParams();
        if (params != null) {
            for (final FormalParamNode p : params) {
                w.string(p.getName().toString());
            }
        }
        w.endArray();
        w.key("body");
        writeExpr(w, def.getBody());
        w.endObject();
    }

    // ------------------------------------------------------------------
    // Expressions. Unknown node kinds are emitted as {"kind":"unhandled",
    // "type": ...} so the bridge never crashes on constructs it does not
    // yet know — they surface explicitly instead.
    // ------------------------------------------------------------------

    static void writeExpr(final JsonWriter w, final SemanticNode node) {
        switch (node.getKind()) {
        case ASTConstants.OpApplKind:
            writeOpAppl(w, (OpApplNode) node);
            break;
        case ASTConstants.NumeralKind:
            // NumeralNode keeps small literals in val() (int) and large ones
            // in bigVal() (BigInteger); useVal() tells which is live. Note
            // toString() returns the raw source image (e.g. "\O7777"), not
            // the numeric value — never use it here.
            final NumeralNode numeral = (NumeralNode) node;
            w.object();
            w.key("kind").string("numeral");
            w.key("value").string(
                    numeral.useVal()
                            ? String.valueOf(numeral.val())
                            : numeral.bigVal().toString());
            w.endObject();
            break;
        case ASTConstants.StringKind:
            final StringNode str = (StringNode) node;
            w.object();
            w.key("kind").string("string");
            w.key("value").string(str.getRep().toString());
            w.endObject();
            break;
        case ASTConstants.LetInKind:
            final LetInNode let = (LetInNode) node;
            w.object();
            w.key("kind").string("letin");
            w.key("defs");
            w.array();
            for (final OpDefNode def : let.getLets()) {
                writeOpDef(w, def);
            }
            w.endArray();
            w.key("body");
            writeExpr(w, let.getBody());
            w.endObject();
            break;
        default:
            w.object();
            w.key("kind").string("unhandled");
            w.key("type").string(node.getClass().getSimpleName());
            w.endObject();
        }
    }

    static void writeOpAppl(final JsonWriter w, final OpApplNode appl) {
        w.object();
        w.key("kind").string("opappl");
        w.key("operator").string(appl.getOperator().getName().toString());
        w.key("args");
        w.array();
        final ExprOrOpArgNode[] args = appl.getArgs();
        if (args != null) {
            for (final ExprOrOpArgNode arg : args) {
                if (arg instanceof ExprNode) {
                    writeExpr(w, (ExprNode) arg);
                } else {
                    // Operator arguments (e.g. higher-order op params) are
                    // out of scope for now; surface them explicitly.
                    w.object();
                    w.key("kind").string("unhandled");
                    w.key("type").string("oparg");
                    w.endObject();
                }
            }
        }
        w.endArray();
        w.endObject();
    }

    static String kindName(final int kind) {
        return kind >= 0 && kind < ASTConstants.kinds.length
                ? ASTConstants.kinds[kind]
                : "kind-" + kind;
    }

    static void printUsage(final PrintStream out) {
        out.println("Usage: SanyJson <spec.tla> [includeDir...]");
        out.println("Parses a TLA+ spec with SANY and emits the resolved");
        out.println("semantic tree as JSON (schema tla-ast/v1) on stdout.");
    }

    // ------------------------------------------------------------------
    // Minimal hand-rolled JSON writer (no external dependencies).
    //
    // State machine: each open container (object or array) tracks whether a
    // comma is pending before its next *entry*. For objects, entries begin
    // at key(); for arrays, at each element value. A value that follows a
    // key never receives a separator — the comma belongs to the next key.
    // ------------------------------------------------------------------

    static final class JsonWriter {
        private final StringBuilder buf = new StringBuilder(1 << 16);

        private static final class Ctx {
            boolean array;
            boolean commaPending;
        }

        private final List<Ctx> stack = new ArrayList<>();

        JsonWriter object() {
            beginValue();
            final Ctx ctx = new Ctx();
            ctx.array = false;
            stack.add(ctx);
            buf.append('{');
            return this;
        }

        JsonWriter array() {
            beginValue();
            final Ctx ctx = new Ctx();
            ctx.array = true;
            stack.add(ctx);
            buf.append('[');
            return this;
        }

        JsonWriter endObject() {
            buf.append('}');
            endContainer();
            return this;
        }

        JsonWriter endArray() {
            buf.append(']');
            endContainer();
            return this;
        }

        JsonWriter key(final String k) {
            final Ctx ctx = peek();
            if (ctx.array) {
                throw new IllegalStateException("key() on an array container");
            }
            if (ctx.commaPending) {
                buf.append(',');
                ctx.commaPending = false;
            }
            quote(k);
            buf.append(':');
            return this;
        }

        JsonWriter string(final String s) {
            beginValue();
            quote(s);
            endValue();
            return this;
        }

        JsonWriter number(final int n) {
            beginValue();
            buf.append(n);
            endValue();
            return this;
        }

        /** Separators for a nested object/array belong to the parent entry. */
        private void beginValue() {
            if (stack.isEmpty()) {
                return; // document root
            }
            final Ctx ctx = peek();
            if (ctx.array && ctx.commaPending) {
                buf.append(',');
                ctx.commaPending = false;
            }
        }

        private void endValue() {
            if (!stack.isEmpty()) {
                peek().commaPending = true;
            }
        }

        private void endContainer() {
            if (!stack.isEmpty()) {
                stack.remove(stack.size() - 1);
                endValue(); // the parent entry completes
            }
        }

        private Ctx peek() {
            return stack.get(stack.size() - 1);
        }

        private void quote(final String s) {
            buf.append('"');
            for (int i = 0; i < s.length(); i++) {
                final char c = s.charAt(i);
                switch (c) {
                case '"': buf.append("\\\""); break;
                case '\\': buf.append("\\\\"); break;
                case '\n': buf.append("\\n"); break;
                case '\r': buf.append("\\r"); break;
                case '\t': buf.append("\\t"); break;
                default:
                    if (c < 0x20) {
                        buf.append(String.format("\\u%04x", (int) c));
                    } else {
                        buf.append(c);
                    }
                }
            }
            buf.append('"');
        }

        @Override
        public String toString() {
            return buf.toString();
        }
    }
}
