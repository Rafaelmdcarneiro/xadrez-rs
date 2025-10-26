use ggez::event::{self, EventHandler, KeyCode, KeyMods, MouseButton};
use ggez::graphics::{self, Color, DrawParam, Image, Rect};
use ggez::{Context, ContextBuilder, GameResult};
use std::path::Path;

// Enum para representar as peças
#[derive(Clone, Copy, Debug, PartialEq)]
enum PecaTipo {
    Peao,
    Torre,
    Cavalo,
    Bispo,
    Rainha,
    Rei,
}

// Enum para representar a cor das peças
#[derive(Clone, Copy, Debug, PartialEq)]
enum Cor {
    Branco,
    Preto,
}

// Struct para representar uma peça
#[derive(Clone, Copy, Debug)]
struct Peca {
    tipo: PecaTipo,
    cor: Cor,
}

// Struct para representar uma posição no tabuleiro
#[derive(Clone, Copy, Debug, PartialEq)]
struct Posicao {
    linha: usize,
    coluna: usize,
}

// Struct principal do jogo
struct Xadrez {
    tabuleiro: [[Option<Peca>; 8]; 8],
    selecionada: Option<Posicao>,
    turno: Cor,
    movimentos_validos: Vec<Posicao>,
}

impl Xadrez {
    fn new() -> GameResult<Xadrez> {
        let mut jogo = Xadrez {
            tabuleiro: [[None; 8]; 8],
            selecionada: None,
            turno: Cor::Branco,
            movimentos_validos: Vec::new(),
        };

        jogo.inicializar_tabuleiro();
        Ok(jogo)
    }

    fn inicializar_tabuleiro(&mut self) {
        // Inicializar peões
        for coluna in 0..8 {
            self.tabuleiro[1][coluna] = Some(Peca { tipo: PecaTipo::Peao, cor: Cor::Preto });
            self.tabuleiro[6][coluna] = Some(Peca { tipo: PecaTipo::Peao, cor: Cor::Branco });
        }

        // Inicializar outras peças
        let pecas_traseiras = [
            PecaTipo::Torre,
            PecaTipo::Cavalo,
            PecaTipo::Bispo,
            PecaTipo::Rainha,
            PecaTipo::Rei,
            PecaTipo::Bispo,
            PecaTipo::Cavalo,
            PecaTipo::Torre,
        ];

        for (coluna, &tipo) in pecas_traseiras.iter().enumerate() {
            self.tabuleiro[0][coluna] = Some(Peca { tipo, cor: Cor::Preto });
            self.tabuleiro[7][coluna] = Some(Peca { tipo, cor: Cor::Branco });
        }
    }

    fn posicao_para_coordenadas(&self, pos: Posicao) -> (f32, f32) {
        (pos.coluna as f32 * 80.0, pos.linha as f32 * 80.0)
    }

    fn coordenadas_para_posicao(&self, x: f32, y: f32) -> Option<Posicao> {
        let coluna = (x / 80.0).floor() as usize;
        let linha = (y / 80.0).floor() as usize;

        if linha < 8 && coluna < 8 {
            Some(Posicao { linha, coluna })
        } else {
            None
        }
    }

    fn get_peca(&self, pos: Posicao) -> Option<Peca> {
        self.tabuleiro[pos.linha][pos.coluna]
    }

    fn set_peca(&mut self, pos: Posicao, peca: Option<Peca>) {
        self.tabuleiro[pos.linha][pos.coluna] = peca;
    }

    fn movimento_valido(&self, origem: Posicao, destino: Posicao) -> bool {
        // Verifica se a posição de destino está nos movimentos válidos
        self.movimentos_validos.contains(&destino)
    }

    fn calcular_movimentos_validos(&mut self, pos: Posicao) {
        self.movimentos_validos.clear();

        if let Some(peca) = self.get_peca(pos) {
            match peca.tipo {
                PecaTipo::Peao => self.calcular_movimentos_peao(pos, peca.cor),
                PecaTipo::Torre => self.calcular_movimentos_torre(pos, peca.cor),
                PecaTipo::Cavalo => self.calcular_movimentos_cavalo(pos, peca.cor),
                PecaTipo::Bispo => self.calcular_movimentos_bispo(pos, peca.cor),
                PecaTipo::Rainha => {
                    self.calcular_movimentos_torre(pos, peca.cor);
                    self.calcular_movimentos_bispo(pos, peca.cor);
                },
                PecaTipo::Rei => self.calcular_movimentos_rei(pos, peca.cor),
            }
        }
    }

    fn calcular_movimentos_peao(&mut self, pos: Posicao, cor: Cor) {
        let direcao = match cor {
            Cor::Branco => -1,
            Cor::Preto => 1,
        };

        // Movimento para frente
        let nova_linha = pos.linha as i32 + direcao;
        if nova_linha >= 0 && nova_linha < 8 {
            let nova_pos = Posicao { linha: nova_linha as usize, coluna: pos.coluna };
            if self.get_peca(nova_pos).is_none() {
                self.movimentos_validos.push(nova_pos);

                // Movimento inicial de duas casas
                let linha_inicial = match cor {
                    Cor::Branco => 6,
                    Cor::Preto => 1,
                };

                if pos.linha == linha_inicial {
                    let nova_linha_2 = nova_linha + direcao;
                    if nova_linha_2 >= 0 && nova_linha_2 < 8 {
                        let nova_pos_2 = Posicao { linha: nova_linha_2 as usize, coluna: pos.coluna };
                        if self.get_peca(nova_pos_2).is_none() {
                            self.movimentos_validos.push(nova_pos_2);
                        }
                    }
                }
            }
        }

        // Capturas diagonais
        for coluna_offset in [-1, 1] {
            let nova_coluna = pos.coluna as i32 + coluna_offset;
            let nova_linha = pos.linha as i32 + direcao;

            if nova_coluna >= 0 && nova_coluna < 8 && nova_linha >= 0 && nova_linha < 8 {
                let nova_pos = Posicao { linha: nova_linha as usize, coluna: nova_coluna as usize };
                if let Some(peca_destino) = self.get_peca(nova_pos) {
                    if peca_destino.cor != cor {
                        self.movimentos_validos.push(nova_pos);
                    }
                }
            }
        }
    }

    fn calcular_movimentos_torre(&mut self, pos: Posicao, cor: Cor) {
        let direcoes = [(0, 1), (1, 0), (0, -1), (-1, 0)];

        for (dl, dc) in direcoes.iter() {
            let mut linha_atual = pos.linha as i32 + dl;
            let mut coluna_atual = pos.coluna as i32 + dc;

            while linha_atual >= 0 && linha_atual < 8 && coluna_atual >= 0 && coluna_atual < 8 {
                let nova_pos = Posicao { linha: linha_atual as usize, coluna: coluna_atual as usize };

                match self.get_peca(nova_pos) {
                    None => {
                        self.movimentos_validos.push(nova_pos);
                    },
                    Some(peca_destino) => {
                        if peca_destino.cor != cor {
                            self.movimentos_validos.push(nova_pos);
                        }
                        break;
                    }
                }

                linha_atual += dl;
                coluna_atual += dc;
            }
        }
    }

    fn calcular_movimentos_cavalo(&mut self, pos: Posicao, cor: Cor) {
        let movimentos = [
            (-2, -1), (-2, 1), (-1, -2), (-1, 2),
            (1, -2), (1, 2), (2, -1), (2, 1)
        ];

        for (dl, dc) in movimentos.iter() {
            let nova_linha = pos.linha as i32 + dl;
            let nova_coluna = pos.coluna as i32 + dc;

            if nova_linha >= 0 && nova_linha < 8 && nova_coluna >= 0 && nova_coluna < 8 {
                let nova_pos = Posicao { linha: nova_linha as usize, coluna: nova_coluna as usize };

                match self.get_peca(nova_pos) {
                    None => {
                        self.movimentos_validos.push(nova_pos);
                    },
                    Some(peca_destino) => {
                        if peca_destino.cor != cor {
                            self.movimentos_validos.push(nova_pos);
                        }
                    }
                }
            }
        }
    }

    fn calcular_movimentos_bispo(&mut self, pos: Posicao, cor: Cor) {
        let direcoes = [(-1, -1), (-1, 1), (1, -1), (1, 1)];

        for (dl, dc) in direcoes.iter() {
            let mut linha_atual = pos.linha as i32 + dl;
            let mut coluna_atual = pos.coluna as i32 + dc;

            while linha_atual >= 0 && linha_atual < 8 && coluna_atual >= 0 && coluna_atual < 8 {
                let nova_pos = Posicao { linha: linha_atual as usize, coluna: coluna_atual as usize };

                match self.get_peca(nova_pos) {
                    None => {
                        self.movimentos_validos.push(nova_pos);
                    },
                    Some(peca_destino) => {
                        if peca_destino.cor != cor {
                            self.movimentos_validos.push(nova_pos);
                        }
                        break;
                    }
                }

                linha_atual += dl;
                coluna_atual += dc;
            }
        }
    }

    fn calcular_movimentos_rei(&mut self, pos: Posicao, cor: Cor) {
        let movimentos = [
            (-1, -1), (-1, 0), (-1, 1),
            (0, -1),           (0, 1),
            (1, -1),  (1, 0),  (1, 1)
        ];

        for (dl, dc) in movimentos.iter() {
            let nova_linha = pos.linha as i32 + dl;
            let nova_coluna = pos.coluna as i32 + dc;

            if nova_linha >= 0 && nova_linha < 8 && nova_coluna >= 0 && nova_coluna < 8 {
                let nova_pos = Posicao { linha: nova_linha as usize, coluna: nova_coluna as usize };

                match self.get_peca(nova_pos) {
                    None => {
                        self.movimentos_validos.push(nova_pos);
                    },
                    Some(peca_destino) => {
                        if peca_destino.cor != cor {
                            self.movimentos_validos.push(nova_pos);
                        }
                    }
                }
            }
        }
    }

    fn mover_peca(&mut self, origem: Posicao, destino: Posicao) {
        let peca = self.get_peca(origem);
        self.set_peca(origem, None);
        self.set_peca(destino, peca);
        self.selecionada = None;
        self.movimentos_validos.clear();
        self.turno = match self.turno {
            Cor::Branco => Cor::Preto,
            Cor::Preto => Cor::Branco,
        };
    }
}

impl EventHandler for Xadrez {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, Color::WHITE);

        // Desenhar tabuleiro
        for linha in 0..8 {
            for coluna in 0..8 {
                let cor = if (linha + coluna) % 2 == 0 {
                    Color::new(1.0, 1.0, 1.0, 1.0) // Branco
                } else {
                    Color::new(0.0, 0.0, 0.0, 1.0) // Preto
                };

                let rect = graphics::Mesh::new_rectangle(
                    ctx,
                    graphics::DrawMode::fill(),
                    Rect::new(coluna as f32 * 80.0, linha as f32 * 80.0, 80.0, 80.0),
                    cor,
                )?;
                graphics::draw(ctx, &rect, DrawParam::default())?;
            }
        }

        // Destacar movimentos válidos
        for &pos in &self.movimentos_validos {
            let rect = graphics::Mesh::new_rectangle(
                ctx,
                graphics::DrawMode::fill(),
                Rect::new(pos.coluna as f32 * 80.0, pos.linha as f32 * 80.0, 80.0, 80.0),
                Color::new(0.0, 1.0, 0.0, 0.3),
            )?;
            graphics::draw(ctx, &rect, DrawParam::default())?;
        }

        // Desenhar peças
        for linha in 0..8 {
            for coluna in 0..8 {
                if let Some(peca) = self.tabuleiro[linha][coluna] {
                    let cor_char = match peca.cor {
                        Cor::Branco => 'w',
                        Cor::Preto => 'b',
                    };

                    let peca_char = match peca.tipo {
                        PecaTipo::Peao => 'p',
                        PecaTipo::Torre => 'r',
                        PecaTipo::Cavalo => 'n',
                        PecaTipo::Bispo => 'b',
                        PecaTipo::Rainha => 'q',
                        PecaTipo::Rei => 'k',
                    };

                    let texto = graphics::Text::new(format!("{}{}", cor_char, peca_char));
                    let (x, y) = self.posicao_para_coordenadas(Posicao { linha, coluna });
                    graphics::draw(ctx, &texto, (ggez::mint::Point2 { x: x + 20.0, y: y + 20.0 },))?;
                }
            }
        }

        // Destacar peça selecionada
        if let Some(pos) = self.selecionada {
            let rect = graphics::Mesh::new_rectangle(
                ctx,
                graphics::DrawMode::stroke(3.0),
                Rect::new(pos.coluna as f32 * 80.0, pos.linha as f32 * 80.0, 80.0, 80.0),
                Color::new(1.0, 0.0, 0.0, 1.0),
            )?;
            graphics::draw(ctx, &rect, DrawParam::default())?;
        }

        graphics::present(ctx)
    }

    fn mouse_button_down_event(&mut self, _ctx: &mut Context, button: MouseButton, x: f32, y: f32) {
        if button == MouseButton::Left {
            if let Some(pos) = self.coordenadas_para_posicao(x, y) {
                // Se já tem uma peça selecionada, tenta mover
                if let Some(origem) = self.selecionada {
                    if self.movimento_valido(origem, pos) {
                        self.mover_peca(origem, pos);
                    } else {
                        // Seleciona nova peça se for do turno correto
                        if let Some(peca) = self.get_peca(pos) {
                            if peca.cor == self.turno {
                                self.selecionada = Some(pos);
                                self.calcular_movimentos_validos(pos);
                            } else {
                                self.selecionada = None;
                                self.movimentos_validos.clear();
                            }
                        } else {
                            self.selecionada = None;
                            self.movimentos_validos.clear();
                        }
                    }
                } else {
                    // Seleciona peça se for do turno correto
                    if let Some(peca) = self.get_peca(pos) {
                        if peca.cor == self.turno {
                            self.selecionada = Some(pos);
                            self.calcular_movimentos_validos(pos);
                        }
                    }
                }
            }
        }
    }
}

fn main() -> GameResult {
    let (ctx, event_loop) = &mut ContextBuilder::new("xadrez_rust", "Autor")
        .build()?;

    let jogo = &mut Xadrez::new()?;
    event::run(ctx, event_loop, jogo)
}
