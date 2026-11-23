! RLT-CSTAR-002 harness. Calls the VERBATIM subroutine mpi_setup of
! ucla-roms 0.16.0 Tools-Roms/partit.F (L1493-1597 -> partit_mpi_setup.F).
      program harness_partit
      implicit none
      integer P, T, gnx, npx, node, xr, er
      integer xs(0:4095), xz(0:4095), es(0:4095), ez(0:4095)
      logical we(0:4095), ee(0:4095), se(0:4095), ne(0:4095)
      character*16 a
      call get_command_argument(1, a)
      read(a,*) P
      call get_command_argument(2, a)
      read(a,*) T
      do gnx=1,P
        do npx=1,T
          xr=gnx+2
          er=3
          call mpi_setup(npx,1, xr,er, xs,xz, es,ez, we,ee,se,ne)
          do node=0,npx-1
            write(*,'(5I7)') gnx, npx, node, xz(node), xs(node)
          enddo
        enddo
      enddo
      end
#include "partit_mpi_setup.F"
