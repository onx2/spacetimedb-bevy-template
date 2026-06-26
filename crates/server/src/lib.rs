use spacetimedb::*;

#[spacetimedb::table(accessor = player, public)]
pub struct Player {
    #[primary_key]
    pub identity: Identity,
    pub x: f32,
    pub y: f32,
}

#[spacetimedb::reducer(client_connected)]
pub fn identity_connected(ctx: &ReducerContext) {
    ctx.db.player().insert(Player {
        identity: ctx.sender(),
        x: 0.0,
        y: 0.0,
    });
}

#[spacetimedb::reducer(client_disconnected)]
pub fn identity_disconnected(ctx: &ReducerContext) {
    ctx.db.player().identity().delete(ctx.sender());
}

/// Directly sets the player's position. In a real application, you should validate input
#[reducer]
pub fn move_player(ctx: &ReducerContext, x: f32, y: f32) {
    if let Some(mut player) = ctx.db.player().identity().find(ctx.sender()) {
        player.x = x;
        player.y = y;
        ctx.db.player().identity().update(player);
    }
}
